use std::cell::RefCell;
use std::str::FromStr;
use candid::{candid_method, export_service};

use ic_cdk::{storage, trap};
use ic_cdk::export::{
    candid::CandidType,
    Principal,
    serde::{Deserialize, Serialize},
};
use ic_cdk_macros::*;

#[derive(CandidType, Serialize, Debug)]
struct PublicKeyReply {
    pub public_key: Vec<u8>,
}

#[derive(CandidType, Serialize, Debug)]
struct SignatureReply {
    pub signature: Vec<u8>,
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

thread_local! {
    static KEY: RefCell<Conf> = RefCell::new( Conf {
        price: 25_000_000_000,
        key: "key1".to_string()  //key_1; dfx_test_key; test_key_1
    });
}

#[derive(CandidType, Deserialize, Clone, Debug, Hash, PartialEq)]
pub struct Conf {
    pub key: String,
    pub price: u64
}

#[init]
async fn init(conf: Option<Conf>) -> () {
    match conf {
        Some(conf) => {
            KEY.with(|storage| {
                storage.replace(conf);
            });
        }
        _ => {}
    };
}

#[update]
#[candid_method(update)]
async fn public_key() -> Result<PublicKeyReply, String> {
    let conf = KEY.with(|storage| {
        storage.borrow_mut().clone()
    });

    let key_id = EcdsaKeyId {
        curve: EcdsaCurve::Secp256k1,
        name: conf.key,
    };
    let ic_canister_id = "aaaaa-aa";
    let ic = CanisterId::from_str(&ic_canister_id).unwrap();

    let caller = ic_cdk::caller().as_slice().to_vec();
    let request = ECDSAPublicKey {
        canister_id: None,
        derivation_path: vec![caller],
        //         name: "key_1".to_string(),
        key_id: key_id.clone(),
    };
    let (res, ): (ECDSAPublicKeyReply, ) = ic_cdk::call(ic, "ecdsa_public_key", (request, ))
        .await
        .map_err(|e| format!("Failed to call ecdsa_public_key {}", e.1))?;

    Ok(PublicKeyReply {
        public_key: res.public_key,
    })
}

#[update]
#[candid_method(update)]
async fn sign(message: Vec<u8>) -> Result<SignatureReply, String> {
    assert!(message.len() == 32);


    let conf = KEY.with(|storage| {
        storage.borrow_mut().clone()
    });

    let key_id = EcdsaKeyId {
        curve: EcdsaCurve::Secp256k1,
        //         name: "key_1".to_string(),
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
    let conf: Conf = KEY.with(|c| {
        return c.borrow_mut().clone();
    });
    let pre_upgrade_data = PersistedData { conf: Some(conf) };
    match storage::stable_save((pre_upgrade_data, 0)) {
        Ok(_) => (),
        Err(_) => trap(&format!("Failed to pre_upgrade"))
    }
}

#[derive(Clone, Debug, Deserialize, CandidType)]
pub struct PersistedData {
    pub conf: Option<Conf>,
}

#[post_upgrade]
fn post_upgrade() {
    match storage::stable_restore() {
        Ok(store) => {
            let (post_data, _a): (PersistedData, i32) = store;
            if post_data.conf.is_some() {
                KEY.with(|storage| {
                    storage.replace(post_data.conf.unwrap());
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
