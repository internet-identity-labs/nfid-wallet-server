//! Stores the encrypted global keys of legacy accounts (written by the AWS lambda with `add_kp`)
//! and, since sc-19796, signs their delegations itself: the lambda salts are delivered sealed
//! (`provision_salts`), callers prove their account with the Identity Manager `get_root_certified`
//! response they fetched themselves.
mod certified_root;
mod legacy_signer;
mod provisioning;

use std::cell::RefCell;
use std::collections::HashMap;

use candid::{CandidType, Principal};
use ic_cdk::api::management_canister::main::{canister_status, raw_rand, CanisterIdRecord};
use ic_cdk::api::set_certified_data;
use ic_cdk::{caller, id, init, post_upgrade, pre_upgrade, query, storage, trap, update};
use ic_certified_map::{AsHashTree, RbTree};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

use certified_root::{verify_certified_root, CertifiedRoot, MAINNET_ROOT_KEY_DER};
use legacy_signer::{
    anonymous_public_key, expiration_ns, sign_anonymous_delegation, sign_global_delegation,
    DelegationChain, Salts, StoredKeyPair,
};
use provisioning::SealedSalts;

/// Newer accounts get delegations from the delegation factory.
const LEGACY_ANCHOR_LIMIT: u64 = 200_000_000;
/// Same limit as the delegation factory.
const MAX_DELEGATION_TTL_MS: u64 = 30 * 24 * 3_600_000;

#[derive(CandidType, Serialize, Debug, Deserialize)]
pub struct KeyPair {
    pub public_key: String,
    pub private_key_encrypted: String,
}

#[derive(CandidType, Serialize, Debug)]
pub struct KeyPairResponse {
    pub key_pair: Option<KeyPair>,
    pub princ: String,
}

#[derive(CandidType, Serialize, Debug)]
pub struct KeyPairObject {
    pub key_pair: KeyPair,
    pub created_date: u64,
}

#[derive(CandidType)]
struct CertifiedKeyPairResponse {
    response: KeyPairResponse,
    certificate: Vec<u8>,
    witness: Vec<u8>,
}

#[derive(Clone, Copy, Debug, CandidType, Deserialize, PartialEq, Serialize, Hash)]
pub enum StorageVariant {
    #[serde(rename = "ETH")]
    ETH,
    #[serde(rename = "BTC")]
    BTC,
    #[serde(rename = "II")]
    II,
}

#[derive(CandidType, Deserialize, Clone, Debug, Hash, PartialEq)]
pub struct Conf {
    pub controllers: Option<Vec<Principal>>,
    pub storage: Option<StorageVariant>,
    pub im_canister: Option<String>,
}

/// Secrets of the legacy delegation signing. Persisted next to the keys; absent in the state
/// written by versions before sc-19796, which decodes as `None`.
#[derive(CandidType, Deserialize, Clone, Default)]
pub struct LegacySignerState {
    /// X25519 secret used once to receive the salts.
    pub provisioning_secret: Option<ByteBuf>,
    pub ecdsa_salt: Option<String>,
    pub anonymous_salt: Option<String>,
}

thread_local! {
    static CONFIG: RefCell<Conf> = RefCell::new( Conf {
        controllers: Default::default(),
        storage: None,
        im_canister: None
    });
    static ECDSA_KEYS: RefCell<HashMap<String,KeyPairObject>> = RefCell::new(HashMap::new());
    static TREE: RefCell<RbTree<String, Vec<u8>>> = const { RefCell::new(RbTree::new()) };
    static LEGACY_SIGNER: RefCell<LegacySignerState> = RefCell::new(LegacySignerState::default());
}

// ---------------------------------------------------------------------------------------------
// Key storage (unchanged behavior)
// ---------------------------------------------------------------------------------------------

#[query]
fn get_kp_certified(key: String) -> CertifiedKeyPairResponse {
    trap_if_not_authenticated_admin();
    let witness = get_count_witness(key.clone()).unwrap_or_default();
    let response = ECDSA_KEYS.with(|keys| match keys.borrow().get(&key) {
        None => KeyPairResponse {
            key_pair: None,
            princ: key.clone(),
        },
        Some(kp) => KeyPairResponse {
            key_pair: Some(KeyPair {
                public_key: kp.key_pair.public_key.clone(),
                private_key_encrypted: kp.key_pair.private_key_encrypted.clone(),
            }),
            princ: key,
        },
    });
    let certificate = ic_cdk::api::data_certificate().expect("No data certificate available");
    CertifiedKeyPairResponse {
        response,
        certificate,
        witness,
    }
}

#[init]
fn init(conf: Option<Conf>) {
    if let Some(conf) = conf {
        CONFIG.with(|storage| {
            storage.replace(conf);
        });
    }
}

#[update]
fn reconfig(conf: Conf) {
    trap_if_not_authenticated_admin();
    CONFIG.with(|storage| {
        storage.replace(conf);
    })
}

#[update]
async fn sync_controllers() -> Vec<String> {
    let (status,) = canister_status(CanisterIdRecord { canister_id: id() })
        .await
        .expect("Sync controller function exited unexpectedly: inter-canister call to management canister for canister_status returned an empty result.");
    let controllers = status.settings.controllers;
    CONFIG.with(|c| c.borrow_mut().controllers.replace(controllers.clone()));
    controllers.iter().map(|x| x.to_text()).collect()
}

#[query]
fn get_public_key(root_principal: String) -> Option<String> {
    ECDSA_KEYS.with(|keys| {
        keys.borrow()
            .get(&root_principal)
            .map(|kp| kp.key_pair.public_key.clone())
    })
}

#[update]
async fn get_kp() -> KeyPairResponse {
    let key = match get_root_id().await {
        None => trap("Unauthorised"),
        Some(k) => k,
    };
    ECDSA_KEYS.with(|keys| match keys.borrow().get(&key) {
        None => KeyPairResponse {
            key_pair: None,
            princ: key,
        },
        Some(kp) => KeyPairResponse {
            key_pair: Some(KeyPair {
                public_key: kp.key_pair.public_key.clone(),
                private_key_encrypted: kp.key_pair.private_key_encrypted.clone(),
            }),
            princ: key,
        },
    })
}

#[update]
async fn add_kp(kp: KeyPair) {
    let key = match get_root_id().await {
        None => trap("Unauthorised"),
        Some(k) => k,
    };
    ECDSA_KEYS.with(|k| {
        let mut keys = k.borrow_mut();
        if keys.contains_key(&key) {
            trap(&format!("Already registered {key}"))
        }
        let kkp = KeyPairObject {
            key_pair: KeyPair {
                public_key: kp.public_key.clone(),
                private_key_encrypted: kp.private_key_encrypted.clone(),
            },
            created_date: ic_cdk::api::time(),
        };
        keys.insert(key.clone(), kkp);
        update_certify_keys(
            key,
            KeyPair {
                public_key: kp.public_key,
                private_key_encrypted: kp.private_key_encrypted,
            },
        );
    })
}

#[query]
fn get_principal(payload: Option<String>) -> (String, Option<String>) {
    let principal = ic_cdk::caller().to_text();
    (principal, payload)
}

#[query]
fn get_all_json(from: u32, mut to: u32) -> String {
    trap_if_not_authenticated_admin();
    let mut principal_key_pairs = all_key_pairs();
    principal_key_pairs.sort_by(|a, b| a.created_date.cmp(&b.created_date));
    let len = principal_key_pairs.len() as u32;
    if to > len {
        to = len;
    }
    let resp = &principal_key_pairs[from as usize..to as usize];
    serde_json::to_string(&resp).expect("Failed to serialize the response to JSON")
}

#[query]
fn count() -> u64 {
    trap_if_not_authenticated_admin();
    ECDSA_KEYS.with(|k| k.borrow().len() as u64)
}

// ---------------------------------------------------------------------------------------------
// Legacy delegation signing (sc-19796)
// ---------------------------------------------------------------------------------------------

#[derive(CandidType, Deserialize)]
struct AnonymousDelegationRequest {
    certified_root: CertifiedRoot,
    domain: String,
    session_key: ByteBuf,
    targets: Vec<Principal>,
    delegation_ttl_ms: Option<u64>,
}

#[derive(CandidType, Deserialize)]
struct GlobalDelegationRequest {
    certified_root: CertifiedRoot,
    session_key: ByteBuf,
    targets: Vec<Principal>,
    delegation_ttl_ms: Option<u64>,
}

#[derive(CandidType, Deserialize)]
struct AnonymousPrincipalRequest {
    certified_root: CertifiedRoot,
    domain: String,
}

#[derive(CandidType, Deserialize)]
struct CandidDelegation {
    pubkey: ByteBuf,
    expiration: u64,
    targets: Option<Vec<Principal>>,
}

#[derive(CandidType, Deserialize)]
struct CandidSignedDelegation {
    delegation: CandidDelegation,
    signature: ByteBuf,
}

#[derive(CandidType, Deserialize)]
struct CandidDelegationChain {
    delegations: Vec<CandidSignedDelegation>,
    public_key: ByteBuf,
}

impl From<DelegationChain> for CandidDelegationChain {
    fn from(chain: DelegationChain) -> Self {
        CandidDelegationChain {
            delegations: chain
                .delegations
                .into_iter()
                .map(|signed| CandidSignedDelegation {
                    delegation: CandidDelegation {
                        pubkey: ByteBuf::from(signed.delegation.pubkey),
                        expiration: signed.delegation.expiration,
                        targets: signed.delegation.targets,
                    },
                    signature: ByteBuf::from(signed.signature),
                })
                .collect(),
            public_key: ByteBuf::from(chain.public_key),
        }
    }
}

/// Subset of the Identity Manager `get_config` response.
#[derive(CandidType, Deserialize)]
struct IdentityManagerConfig {
    lambda: Option<Principal>,
}

#[derive(CandidType, Deserialize)]
struct LegacySignerStatus {
    im_canister: Option<String>,
    salts_provisioned: bool,
    salts_fingerprint: Option<String>,
    provisioning_key_pending: bool,
    global_keys: u64,
}

#[update]
async fn get_global_delegation(
    request: GlobalDelegationRequest,
) -> Result<CandidDelegationChain, String> {
    let root = authorize(&request.certified_root).await?;
    let salts = salts().ok_or("Salts are not provisioned")?;
    let stored = stored_key(&root).ok_or("No global key for this account")?;
    sign_global_delegation(
        &root,
        &stored,
        &salts.ecdsa_salt,
        &request.session_key,
        &request.targets,
        expiration(request.delegation_ttl_ms)?,
    )
    .map(Into::into)
    .map_err(|e| e.to_string())
}

#[update]
async fn get_anonymous_delegation(
    request: AnonymousDelegationRequest,
) -> Result<CandidDelegationChain, String> {
    let root = authorize(&request.certified_root).await?;
    let salts = salts().ok_or("Salts are not provisioned")?;
    sign_anonymous_delegation(
        &root,
        &request.domain,
        &salts,
        &request.session_key,
        &request.targets,
        expiration(request.delegation_ttl_ms)?,
    )
    .map(Into::into)
    .map_err(|e| e.to_string())
}

#[update]
async fn get_anonymous_principal(request: AnonymousPrincipalRequest) -> Result<Principal, String> {
    let root = authorize(&request.certified_root).await?;
    let salts = salts().ok_or("Salts are not provisioned")?;
    Ok(Principal::self_authenticating(anonymous_public_key(
        &root,
        &request.domain,
        &salts,
    )))
}

/// Returns the X25519 key to seal the salts for `provision_salts`, creating it when needed.
#[update]
async fn get_provisioning_key() -> Result<String, String> {
    require_migration_access().await?;
    if LEGACY_SIGNER.with(|s| s.borrow().provisioning_secret.is_none()) {
        let (random,) = raw_rand()
            .await
            .map_err(|(code, message)| format!("raw_rand failed: {code:?} {message}"))?;
        LEGACY_SIGNER.with(|s| {
            s.borrow_mut()
                .provisioning_secret
                .get_or_insert(ByteBuf::from(random));
        });
    }
    Ok(hex::encode(provisioning::public_key(&provisioning_secret()?)))
}

/// Loads the lambda salts sealed to `get_provisioning_key`; returns their fingerprint, so the caller
/// can check them without reading the state.
#[update]
async fn provision_salts(sealed: SealedSalts) -> Result<String, String> {
    // Loading them once is enough; replacing them would change every anonymous principal, so only a
    // controller may do it again.
    if salts().is_some() {
        require_controller().map_err(|_| "Salts are already provisioned".to_string())?;
    } else {
        require_migration_access().await?;
    }
    let salts = provisioning::open_salts(&provisioning_secret()?, &id(), &sealed)?;
    let fingerprint = provisioning::fingerprint(&salts);
    LEGACY_SIGNER.with(|s| {
        let mut state = s.borrow_mut();
        state.ecdsa_salt = Some(salts.ecdsa_salt);
        state.anonymous_salt = Some(salts.anonymous_salt);
        state.provisioning_secret = None;
    });
    Ok(fingerprint)
}

#[query]
fn legacy_signer_status() -> Result<LegacySignerStatus, String> {
    require_controller()?;
    let state = LEGACY_SIGNER.with(|s| s.borrow().clone());
    Ok(LegacySignerStatus {
        im_canister: CONFIG.with(|c| c.borrow().im_canister.clone()),
        salts_provisioned: salts().is_some(),
        salts_fingerprint: salts().as_ref().map(provisioning::fingerprint),
        provisioning_key_pending: state.provisioning_secret.is_some(),
        global_keys: ECDSA_KEYS.with(|k| k.borrow().len() as u64),
    })
}

/// Verifies the caller's certified root and that the account is a legacy one.
async fn authorize(certified_root: &CertifiedRoot) -> Result<String, String> {
    let caller = caller();
    if caller == Principal::anonymous() {
        return Err("Anonymous caller".into());
    }
    let im_canister = im_canister()?;
    let root_key = hex::decode(MAINNET_ROOT_KEY_DER).expect("valid constant");
    verify_certified_root(
        &im_canister,
        &root_key,
        &caller,
        certified_root,
        u128::from(ic_cdk::api::time()),
    )?;

    let (anchor,): (Option<u64>,) =
        ic_cdk::call(im_canister, "get_anchor_by_principal", (caller.to_text(),))
            .await
            .map_err(|(code, message)| {
                format!("Identity Manager call failed: {code:?} {message}")
            })?;
    match anchor {
        Some(anchor) if anchor < LEGACY_ANCHOR_LIMIT => Ok(certified_root.root.clone()),
        Some(_) => Err("Not a legacy account".into()),
        None => Err("Unknown access point".into()),
    }
}

fn expiration(delegation_ttl_ms: Option<u64>) -> Result<u64, String> {
    expiration_ns(
        ic_cdk::api::time() / 1_000_000,
        delegation_ttl_ms.map(|ttl| ttl.min(MAX_DELEGATION_TTL_MS)),
    )
    .map_err(|e| e.to_string())
}

/// Migration calls are allowed for controllers and for the delegation lambda: the same principal
/// that signs today, taken from the Identity Manager configuration.
async fn require_migration_access() -> Result<(), String> {
    if require_controller().is_ok() {
        return Ok(());
    }
    let (im_config,): (IdentityManagerConfig,) = ic_cdk::call(im_canister()?, "get_config", ())
        .await
        .map_err(|(code, message)| format!("Identity Manager call failed: {code:?} {message}"))?;
    if im_config.lambda == Some(caller()) {
        Ok(())
    } else {
        Err("Unauthorized".into())
    }
}

fn require_controller() -> Result<(), String> {
    if ic_cdk::api::is_controller(&caller()) {
        Ok(())
    } else {
        Err("Unauthorized".into())
    }
}

fn im_canister() -> Result<Principal, String> {
    let text = CONFIG
        .with(|c| c.borrow().im_canister.clone())
        .ok_or("The Identity Manager canister is not configured")?;
    Principal::from_text(text).map_err(|_| "Invalid Identity Manager canister id".into())
}

fn salts() -> Option<Salts> {
    LEGACY_SIGNER.with(|s| {
        let state = s.borrow();
        Some(Salts {
            ecdsa_salt: state.ecdsa_salt.clone()?,
            anonymous_salt: state.anonymous_salt.clone()?,
        })
    })
}

fn provisioning_secret() -> Result<[u8; 32], String> {
    LEGACY_SIGNER
        .with(|s| s.borrow().provisioning_secret.clone())
        .and_then(|secret| secret.into_vec().try_into().ok())
        .ok_or_else(|| "Call get_provisioning_key first".into())
}

fn stored_key(root: &str) -> Option<StoredKeyPair> {
    ECDSA_KEYS.with(|keys| {
        keys.borrow().get(root).map(|kp| StoredKeyPair {
            public_key: kp.key_pair.public_key.clone(),
            private_key_encrypted: kp.key_pair.private_key_encrypted.clone(),
        })
    })
}

// ---------------------------------------------------------------------------------------------
// Upgrades
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Deserialize, CandidType)]
pub struct PersistedData {
    pub conf: Option<Conf>,
    pub keys: Option<Vec<PrincipalKP>>,
    /// Added in sc-19796; older versions ignore it, older states lack it.
    pub legacy_signer: Option<LegacySignerState>,
}

#[derive(CandidType, Deserialize, Clone, Debug, Hash, PartialEq, Serialize)]
pub struct PrincipalKP {
    pub public_key: String,
    pub private_key: String,
    pub principal: String,
    pub created_date: u64,
}

#[pre_upgrade]
fn pre_upgrade() {
    let conf: Conf = CONFIG.with(|c| c.borrow().clone());
    let pre_upgrade_data = PersistedData {
        conf: Some(conf),
        keys: Some(all_key_pairs()),
        legacy_signer: Some(LEGACY_SIGNER.with(|s| s.borrow().clone())),
    };
    if storage::stable_save((pre_upgrade_data, 0i32)).is_err() {
        trap("Failed to pre_upgrade")
    }
}

#[post_upgrade]
fn post_upgrade() {
    let (post_data, _): (PersistedData, i32) = match storage::stable_restore() {
        Ok(store) => store,
        Err(message) => trap(message.as_str()),
    };
    if let Some(conf) = post_data.conf {
        CONFIG.with(|storage| {
            storage.replace(conf);
        });
    }
    if let Some(state) = post_data.legacy_signer {
        LEGACY_SIGNER.with(|s| s.replace(state));
    }
    if let Some(keys) = post_data.keys {
        let mut kpp = HashMap::default();
        let mut tree: RbTree<String, Vec<u8>> = RbTree::new();
        for x in keys {
            tree.insert(
                x.principal.clone(),
                key_pair_hash(&x.public_key, &x.private_key),
            );
            kpp.insert(
                x.principal,
                KeyPairObject {
                    key_pair: KeyPair {
                        public_key: x.public_key,
                        private_key_encrypted: x.private_key,
                    },
                    created_date: x.created_date,
                },
            );
        }
        TREE.with(|tr| {
            set_certified_data(&tree.root_hash());
            tr.replace(tree);
        });
        ECDSA_KEYS.with(|storage| storage.replace(kpp));
    }
}

fn all_key_pairs() -> Vec<PrincipalKP> {
    ECDSA_KEYS.with(|k| {
        k.borrow()
            .iter()
            .map(|(principal, kp)| PrincipalKP {
                public_key: kp.key_pair.public_key.clone(),
                private_key: kp.key_pair.private_key_encrypted.clone(),
                created_date: kp.created_date,
                principal: principal.clone(),
            })
            .collect()
    })
}

fn trap_if_not_authenticated_admin() {
    let princ = caller();
    match CONFIG.with(|c| c.borrow().controllers.clone()) {
        None => trap("Unauthorised"),
        Some(controllers) => {
            if !controllers.contains(&princ) {
                trap("Unauthorised")
            }
        }
    }
}

async fn get_root_id() -> Option<String> {
    match CONFIG.with(|c| c.borrow().im_canister.clone()) {
        None => {
            Some(caller().to_text()) //DONE FOR TESTING PURPOSES
        }
        Some(canister) => {
            let princ = caller();
            let im_canister =
                Principal::from_text(canister).expect("Unable to obtain Principal from im_canister.");
            let res: Option<String> =
                match ic_cdk::call(im_canister, "get_root_by_principal", (princ.to_text(), 0)).await
                {
                    Ok((res,)) => res,
                    Err((_, err)) => trap(&format!("failed to request IM: {}", err)),
                };
            res
        }
    }
}

fn get_count_witness(key: String) -> anyhow::Result<Vec<u8>> {
    TREE.with(|tree| {
        let tree = tree.borrow();
        let mut witness = vec![];
        let mut witness_serializer = serde_cbor::Serializer::new(&mut witness);
        witness_serializer.self_describe()?;
        tree.witness(key.as_bytes())
            .serialize(&mut witness_serializer)
            .expect("Failed to serialize the witness for the given key.");
        Ok(witness)
    })
}

fn update_certify_keys(key: String, kp: KeyPair) -> String {
    TREE.with(|k| {
        let mut keys = k.borrow_mut();
        keys.insert(
            key.clone(),
            key_pair_hash(&kp.public_key, &kp.private_key_encrypted),
        );
        set_certified_data(&keys.root_hash());
        key
    })
}

fn key_pair_hash(public_key: &str, private_key_encrypted: &str) -> Vec<u8> {
    let new_owned_string = public_key.to_string() + private_key_encrypted;
    hex::decode(sha256::digest(new_owned_string))
        .expect("Failed to decode the hex string from the SHA256 digest")
}

ic_cdk::export_candid!();

#[cfg(test)]
mod tests {
    use super::*;
    use candid::{Decode, Encode};

    /// `PersistedData` as written by the version before sc-19796.
    #[derive(CandidType)]
    struct OldPersistedData {
        conf: Option<Conf>,
        keys: Option<Vec<PrincipalKP>>,
    }

    fn key(n: u64) -> PrincipalKP {
        PrincipalKP {
            public_key: format!("pk{n}"),
            private_key: format!("sk{n}"),
            principal: format!("root{n}"),
            created_date: n,
        }
    }

    #[test]
    fn decodes_the_state_of_the_previous_version() {
        let conf = Conf {
            controllers: Some(vec![Principal::anonymous()]),
            storage: Some(StorageVariant::II),
            im_canister: Some("74gpt-tiaaa-aaaak-aacaa-cai".into()),
        };
        let old = OldPersistedData {
            conf: Some(conf.clone()),
            keys: Some(vec![key(1), key(2)]),
        };
        let bytes = Encode!(&old, &0i32).unwrap();

        let (decoded, marker) = Decode!(&bytes, PersistedData, i32).unwrap();

        assert_eq!(marker, 0);
        assert_eq!(decoded.conf, Some(conf));
        assert_eq!(decoded.keys, Some(vec![key(1), key(2)]));
        assert!(decoded.legacy_signer.is_none());
    }

    #[test]
    fn the_previous_version_decodes_the_new_state() {
        #[derive(Deserialize, CandidType)]
        struct OldDecoded {
            conf: Option<Conf>,
            keys: Option<Vec<PrincipalKP>>,
        }
        let new = PersistedData {
            conf: None,
            keys: Some(vec![key(7)]),
            legacy_signer: Some(LegacySignerState {
                provisioning_secret: None,
                ecdsa_salt: Some("a".into()),
                anonymous_salt: Some("b".into()),
            }),
        };
        let bytes = Encode!(&new, &0i32).unwrap();

        let (decoded, _) = Decode!(&bytes, OldDecoded, i32).unwrap();

        assert_eq!(decoded.keys, Some(vec![key(7)]));
    }

    #[test]
    fn candid_interface_matches_did_file() {
        use candid_parser::utils::{service_equal, CandidSource};
        let did = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ecdsa_signer.did");
        service_equal(
            CandidSource::Text(&super::__export_service()),
            CandidSource::File(&did),
        )
        .expect("ecdsa_signer.did does not match the implementation");
    }
}
