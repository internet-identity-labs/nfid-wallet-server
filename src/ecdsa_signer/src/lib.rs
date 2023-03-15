use std::cell::RefCell;
use std::collections::HashMap;
use std::str;
use std::str::FromStr;
use std::time::Duration;

use candid::{candid_method, export_service};
use ic_cdk::{storage, trap};
use ic_cdk::export::{
    candid::CandidType,
    Principal,
    serde::{Deserialize, Serialize},
};
use ic_cdk_macros::*;
use structure::ttlhashmap::TtlHashMap;

mod structure;

#[derive(CandidType, Serialize, Debug)]
pub struct PublicKeyReply {
    pub public_key: Vec<u8>,
}

#[derive(CandidType, Serialize, Debug, Deserialize)]
pub struct KeyPair {
    pub public_key: String,
    pub private_key_encrypted: String,
}

#[derive(CandidType, Serialize, Debug)]
pub struct KeyPairResponse {
    pub key_pair: Option<KeyPair>,
}

#[derive(CandidType, Serialize, Debug)]
pub struct SignatureReply {
    pub signature: Vec<u8>,
}

#[derive(CandidType, Serialize, Debug)]
pub struct SignatureAsset {
    pub signature: Vec<u8>,
    pub timestamp: Vec<u8>,
}

type CanisterId = Principal;

#[derive(CandidType, Serialize, Debug)]
struct ECDSAPublicKey {
    pub canister_id: Option<CanisterId>,
    pub derivation_path: Vec<Vec<u8>>,
    pub key_id: EcdsaKeyId,
}

#[derive(CandidType, Deserialize, Debug)]
struct ECDSAPublicKeyReply {
    pub public_key: Vec<u8>,
    pub chain_code: Vec<u8>,
}

#[derive(CandidType, Serialize, Debug)]
struct SignWithECDSA {
    pub message_hash: Vec<u8>,
    pub derivation_path: Vec<Vec<u8>>,
    pub key_id: EcdsaKeyId,
}

#[derive(CandidType, Deserialize, Debug)]
struct SignWithECDSAReply {
    pub signature: Vec<u8>,
}

#[derive(CandidType, Serialize, Debug, Clone)]
struct EcdsaKeyId {
    pub curve: EcdsaCurve,
    pub name: String,
}

#[derive(CandidType, Serialize, Debug, Clone)]
pub enum EcdsaCurve {
    #[serde(rename = "secp256k1")]
    Secp256k1,
}

const DEFAULT_TOKEN_TTL: u64 = 300;

thread_local! {
    static CONFIG: RefCell<Conf> = RefCell::new( Conf {
        price: 23_000_000_000,
        key: "test_key_1".to_string(),  //key_1; dfx_test_key; test_key_1
        ttl: 300
    });
    pub static SIGNATURES: RefCell<TtlHashMap<String,SignatureReply>> = RefCell::new(TtlHashMap::new(Duration::from_secs(DEFAULT_TOKEN_TTL)));
    pub static KEYS: RefCell<HashMap<String,PublicKeyReply>> = RefCell::new(HashMap::new());
    pub static ECDSA_KEYS: RefCell<HashMap<String,KeyPair>> = RefCell::new(HashMap::new());
}

#[derive(CandidType, Deserialize, Clone, Debug, Hash, PartialEq)]
pub struct Conf {
    pub key: String,
    pub price: u64,
    pub ttl: u64,
}

#[init]
async fn init(conf: Option<Conf>) -> () {
    match conf {
        Some(conf) => {
            SIGNATURES.with(|signatures| {
                signatures.borrow_mut().ttl = Duration::from_secs(conf.ttl.clone());
            });
            CONFIG.with(|storage| {
                storage.replace(conf);
            });
        }
        _ => {}
    };
}


#[query]
#[candid_method(query)]
async fn get_kp() -> KeyPairResponse {
    let principal = ic_cdk::caller().to_text();
    ECDSA_KEYS.with(|keys| {
        match keys.borrow().get(&principal) {
            None => {
                KeyPairResponse { key_pair: None }
            }
            Some(kp) => {
                let response = KeyPairResponse {
                    key_pair: Some(KeyPair {
                        public_key: kp.public_key.clone(),
                        private_key_encrypted: kp.private_key_encrypted.clone(),
                    })
                };
                response
            }
        }
    })
}

#[update]
#[candid_method(update)]
async fn add_kp(kp: KeyPair) {
    let principal = ic_cdk::caller().to_text();
    ECDSA_KEYS.with(|k| {
        let mut keys = k.borrow_mut();
        if keys.contains_key(&principal) {
            trap( &format!("Already registered {}", principal))
        }
        keys.insert(principal, kp);
    })
}

#[query]
#[candid_method(query)]
async fn get_principal(payload: Option<String>) -> (String, Option<String>) {
    let principal = ic_cdk::caller().to_text();
    (principal, payload)
}

#[update]
#[candid_method(update)]
async fn public_key() -> Result<PublicKeyReply, String> {
    let conf = CONFIG.with(|storage| {
        storage.borrow_mut().clone()
    });

    let key_id = EcdsaKeyId {
        curve: EcdsaCurve::Secp256k1,
        name: conf.key,
    };
    let ic_canister_id = "aaaaa-aa";
    let ic = CanisterId::from_str(&ic_canister_id).unwrap();

    let principal = ic_cdk::caller().to_text();
    let derivation_path = ic_cdk::caller().as_slice().to_vec();

    let request = ECDSAPublicKey {
        canister_id: None,
        derivation_path: vec![derivation_path],
        key_id: key_id.clone(),
    };

    let saved_key_reply = KEYS.with(|keys| {
        let k = keys.borrow_mut();
        match k.get(&principal) {
            None => { None }
            Some(kr) => {
                SIGNATURES.with(|signatures| {
                    signatures.borrow_mut().cleanup();
                });
                Some(kr.public_key.clone())
            }
        }
    });

    match saved_key_reply {
        None => {
            let (res, ): (ECDSAPublicKeyReply, ) = ic_cdk::call(ic, "ecdsa_public_key", (request, ))
                .await
                .map_err(|e| format!("Failed to call ecdsa_public_key {}", e.1))?;

            KEYS.with(|keys| {
                keys.borrow_mut().insert(principal, PublicKeyReply {
                    public_key: res.public_key.clone(),
                })
            });

            Ok(PublicKeyReply {
                public_key: res.public_key,
            })
        }
        Some(key) => {
            Ok(PublicKeyReply {
                public_key: key,
            })
        }
    }
}

#[update]
#[candid_method(update)]
async fn prepare_signature(message: Vec<u8>) -> String {
    match sign(message.clone()).await {
        Ok(signature_reply) => {
            let hex = hex::encode(&message);
            SIGNATURES.with(|signatures| {
                signatures.borrow_mut().insert(hex.clone(), signature_reply)
            });
            hex
        }
        Err(err) => {
            trap(&err)
        }
    }
}

#[query]
#[candid_method(query)]
async fn get_signature(key: String) -> Result<SignatureReply, String> {
    SIGNATURES.with(|signatures| {
        match signatures.borrow_mut().get(&key) {
            None => {
                Err(String::from("No such signature"))
            }
            Some(reply) => {
                Ok(SignatureReply {
                    signature: reply.signature.clone(),
                })
            }
        }
    })
}

#[update]
#[candid_method(update)]
async fn sign(message: Vec<u8>) -> Result<SignatureReply, String> {
    assert!(message.len() == 32);

    let conf = CONFIG.with(|storage| {
        storage.borrow_mut().clone()
    });

    let key_id = EcdsaKeyId {
        curve: EcdsaCurve::Secp256k1,
        name: conf.key,
    };
    let ic_canister_id = "aaaaa-aa";
    let ic = CanisterId::from_str(&ic_canister_id).unwrap();

    let caller = ic_cdk::caller().as_slice().to_vec();
    let request = SignWithECDSA {
        message_hash: message.clone(),
        derivation_path: vec![caller],
        key_id,
    };
    let (res, ): (SignWithECDSAReply, ) =
        ic_cdk::api::call::call_with_payment(ic, "sign_with_ecdsa", (request, ), conf.price)
            .await
            .map_err(|e| format!("Failed to call sign_with_ecdsa {}", e.1))?;

    Ok(SignatureReply {
        signature: res.signature,
    })
}


#[pre_upgrade]
fn pre_upgrade() {
    let conf: Conf = CONFIG.with(|c| {
        return c.borrow_mut().clone();
    });
    let principal_key_pairs = ECDSA_KEYS.with(|k| {
        let pkp: Vec<PrincipalKP> = k.borrow_mut()
            .iter()
            .map(|a| PrincipalKP {
                public_key: a.1.public_key.clone(),
                private_key: a.1.private_key_encrypted.clone(),
                principal: a.0.clone(),
            })
            .collect();
        pkp
    });

    let pre_upgrade_data = PersistedData { conf: Some(conf), keys: Some(principal_key_pairs) };
    match storage::stable_save((pre_upgrade_data, 0)) {
        Ok(_) => (),
        Err(_) => trap(&format!("Failed to pre_upgrade"))
    }
}

#[derive(Clone, Debug, Deserialize, CandidType)]
pub struct PersistedData {
    pub conf: Option<Conf>,
    pub keys: Option<Vec<PrincipalKP>>,
}

#[derive(CandidType, Deserialize, Clone, Debug, Hash, PartialEq)]
pub struct PrincipalKP {
    pub public_key: String,
    pub private_key: String,
    pub principal: String,
}

#[post_upgrade]
fn post_upgrade() {
    match storage::stable_restore() {
        Ok(store) => {
            let (post_data, _a): (PersistedData, i32) = store;
            if post_data.conf.is_some() {
                CONFIG.with(|storage| {
                    storage.replace(post_data.conf.clone().unwrap());
                });
            };
            if post_data.keys.is_some() {
                ECDSA_KEYS.with(|storage| {
                    let mut kpp = HashMap::default();
                    for x in post_data.keys.unwrap().into_iter() {
                        kpp.insert(x.principal, KeyPair {
                            public_key: x.public_key,
                            private_key_encrypted: x.private_key,
                        });
                    };
                    storage.replace(kpp);
                });
            }
        }
        Err(message) => trap(message.as_str())
    }
}

export_service!();

#[ic_cdk_macros::query(name = "__get_candid_interface")]
fn export_candid() -> String {
    __export_service()
}
