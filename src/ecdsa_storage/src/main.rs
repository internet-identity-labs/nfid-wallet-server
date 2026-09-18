//! Signs delegations for legacy accounts (anchor < 200_000_000) instead of the AWS lambda
//! (`sms-sender-serverless`): same keys, same principals, no key generation.
//!
//! Callers prove their account with the Identity Manager `get_root_certified` response they fetched
//! themselves; the lambda salts are delivered encrypted (see `provisioning`) and global keys are
//! imported from `signer_ic`.

mod certified_root;
mod legacy_signer;
mod provisioning;
mod state;

use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api::management_canister::main::raw_rand;
use ic_cdk::{init, post_upgrade, query, update};
use serde_bytes::ByteBuf;

use certified_root::{verify_certified_root, CertifiedRoot, MAINNET_ROOT_KEY_DER};
use legacy_signer::{
    anonymous_public_key, expiration_ns, sign_anonymous_delegation, sign_global_delegation,
    DelegationChain,
};
use provisioning::SealedSalts;
use state::GlobalKey;

/// Newer accounts get delegations from the delegation factory.
const LEGACY_ANCHOR_LIMIT: u64 = 200_000_000;
/// Same limit as the delegation factory.
const MAX_DELEGATION_TTL_MS: u64 = 30 * 24 * 3_600_000;

#[derive(CandidType, Deserialize)]
struct InitArgs {
    im_canister: Principal,
    /// For local replicas only; the mainnet key is used otherwise.
    ic_root_key: Option<ByteBuf>,
}

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

#[derive(CandidType, Deserialize)]
struct ImportedKeyPair {
    root: String,
    public_key: String,
    private_key_encrypted: String,
}

#[derive(CandidType, Deserialize)]
struct Status {
    im_canister: Option<Principal>,
    migrator: Option<Principal>,
    custom_ic_root_key: bool,
    salts_provisioned: bool,
    salts_fingerprint: Option<String>,
    provisioning_key_pending: bool,
    global_keys: u64,
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

#[init]
fn init(args: Option<InitArgs>) {
    apply_args(args);
}

#[post_upgrade]
fn post_upgrade(args: Option<InitArgs>) {
    apply_args(args);
}

fn apply_args(args: Option<InitArgs>) {
    if let Some(args) = args {
        state::update_config(|config| {
            config.im_canister = Some(args.im_canister);
            config.ic_root_key = args.ic_root_key.map(ByteBuf::into_vec);
        });
    }
}

#[update]
async fn get_anonymous_delegation(
    request: AnonymousDelegationRequest,
) -> Result<CandidDelegationChain, String> {
    let root = authorize(&request.certified_root).await?;
    let salts = state::salts().ok_or("Salts are not provisioned")?;
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
async fn get_global_delegation(
    request: GlobalDelegationRequest,
) -> Result<CandidDelegationChain, String> {
    let root = authorize(&request.certified_root).await?;
    let salts = state::salts().ok_or("Salts are not provisioned")?;
    let stored = state::global_key(&root).ok_or("No global key for this account")?;
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
async fn get_anonymous_principal(request: AnonymousPrincipalRequest) -> Result<Principal, String> {
    let root = authorize(&request.certified_root).await?;
    let salts = state::salts().ok_or("Salts are not provisioned")?;
    Ok(Principal::self_authenticating(anonymous_public_key(
        &root,
        &request.domain,
        &salts,
    )))
}

/// Returns the X25519 key to seal the salts for `provision_salts`, creating it when needed.
#[update]
async fn get_provisioning_key() -> Result<String, String> {
    require_controller_or_migrator()?;
    if state::config().provisioning_secret.is_none() {
        let (random,) = raw_rand()
            .await
            .map_err(|(code, message)| format!("raw_rand failed: {code:?} {message}"))?;
        state::update_config(|config| {
            config.provisioning_secret.get_or_insert(random);
        });
    }
    let secret = provisioning_secret()?;
    Ok(hex::encode(provisioning::public_key(&secret)))
}

#[update]
fn provision_salts(sealed: SealedSalts) -> Result<(), String> {
    // Loading them once is enough; replacing them would change every anonymous principal, so only a
    // controller may do it again.
    if state::salts().is_some() {
        require_controller().map_err(|_| "Salts are already provisioned".to_string())?;
    } else {
        require_controller_or_migrator()?;
    }
    let salts = provisioning::open_salts(&provisioning_secret()?, &ic_cdk::id(), &sealed)?;
    state::update_config(|config| {
        config.ecdsa_salt = Some(salts.ecdsa_salt);
        config.anonymous_salt = Some(salts.anonymous_salt);
        config.provisioning_secret = None;
    });
    Ok(())
}

/// Copies records of `signer_ic` (`get_all_json`); returns the number of stored keys.
#[update]
fn import_global_keys(keys: Vec<ImportedKeyPair>) -> Result<u64, String> {
    let is_controller = require_controller().is_ok();
    if !is_controller {
        require_controller_or_migrator()?;
    }
    for key in &keys {
        Principal::from_text(&key.root).map_err(|_| format!("Invalid root {}", key.root))?;
        if hex::decode(&key.public_key).is_err() || hex::decode(&key.private_key_encrypted).is_err()
        {
            return Err(format!("Invalid key pair of {}", key.root));
        }
        // Re-importing the same record is fine, replacing a key with another one is not: that would
        // change the user's global principal.
        if let Some(stored) = state::global_key(&key.root) {
            let same = stored.public_key == key.public_key
                && stored.private_key_encrypted == key.private_key_encrypted;
            if !same && !is_controller {
                return Err(format!(
                    "Another global key of {} is already stored",
                    key.root
                ));
            }
        }
    }
    for key in keys {
        state::insert_global_key(
            key.root,
            GlobalKey {
                public_key: key.public_key,
                private_key_encrypted: key.private_key_encrypted,
            },
        );
    }
    Ok(state::global_keys_count())
}

#[query]
fn status() -> Result<Status, String> {
    require_controller_or_migrator()?;
    let config = state::config();
    Ok(Status {
        im_canister: config.im_canister,
        migrator: config.migrator,
        custom_ic_root_key: config.ic_root_key.is_some(),
        salts_provisioned: state::salts().is_some(),
        salts_fingerprint: state::salts().as_ref().map(provisioning::fingerprint),
        provisioning_key_pending: config.provisioning_secret.is_some(),
        global_keys: state::global_keys_count(),
    })
}

/// Verifies the caller's certified root and that the account is a legacy one.
async fn authorize(certified_root: &CertifiedRoot) -> Result<String, String> {
    let caller = ic_cdk::caller();
    if caller == Principal::anonymous() {
        return Err("Anonymous caller".into());
    }
    let config = state::config();
    let im_canister = config
        .im_canister
        .ok_or("The Identity Manager canister is not configured")?;
    let root_key = config
        .ic_root_key
        .unwrap_or_else(|| hex::decode(MAINNET_ROOT_KEY_DER).expect("valid constant"));
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

/// Grants or revokes the migration role (the lambda that loads the salts and imports the keys).
#[update]
fn set_migrator(migrator: Option<Principal>) -> Result<(), String> {
    require_controller()?;
    state::update_config(|config| config.migrator = migrator);
    Ok(())
}

fn require_controller_or_migrator() -> Result<(), String> {
    let caller = ic_cdk::caller();
    if ic_cdk::api::is_controller(&caller) || state::config().migrator == Some(caller) {
        Ok(())
    } else {
        Err("Unauthorized".into())
    }
}

fn require_controller() -> Result<(), String> {
    if ic_cdk::api::is_controller(&ic_cdk::caller()) {
        Ok(())
    } else {
        Err("Unauthorized".into())
    }
}

fn provisioning_secret() -> Result<[u8; 32], String> {
    state::config()
        .provisioning_secret
        .and_then(|secret| secret.try_into().ok())
        .ok_or_else(|| "Call get_provisioning_key first".into())
}

ic_cdk::export_candid!();

fn main() {}

#[cfg(test)]
mod tests {
    use candid_parser::utils::{service_equal, CandidSource};
    use std::path::Path;

    #[test]
    fn candid_interface_matches_did_file() {
        let did = Path::new(env!("CARGO_MANIFEST_DIR")).join("ecdsa_storage.did");
        service_equal(
            CandidSource::Text(&super::__export_service()),
            CandidSource::File(&did),
        )
        .unwrap();
    }
}
