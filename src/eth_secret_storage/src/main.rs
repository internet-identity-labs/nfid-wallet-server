use std::convert::TryInto;
use std::str::FromStr;

use canister_api_macros::collect_metrics;
use ethers_core::k256::sha2::{Sha256, Digest};
use ic_cdk::export::Principal;
use ic_cdk::export::candid::CandidType;
use ic_cdk::{trap, storage, call};
use serde::Deserialize;
use ic_cdk_macros::{pre_upgrade, post_upgrade, query, update};

use std::cell::RefCell;
use std::collections::HashSet;
use ethers_core::types::Signature;

const MESSAGE: &str = "Hi there from NFID! Sign this message to prove you own this wallet and we’ll log you in. This won’t cost you any Ether.";

thread_local! {
    static CONTROLLERS: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
    static KEY: RefCell<Option<String>> = RefCell::new(None);
}

#[derive(Clone, Debug, Deserialize, CandidType)]
pub struct PersistedData {
    pub key: Option<String>
}

#[update]
async fn init() -> () {
    let is_exist = KEY.with(|storage| { 
        storage.borrow().is_some()
    });

    if is_exist {
        return ();
    }

    let key: String = match call(Principal::management_canister(), "raw_rand", ()).await {
        Ok((result, )) => {
            let bytes: Vec<u8> = result;
            hex::encode(bytes)
        },
        Err((_, err)) => trap(&format!("Failed to get salt: {}", err)),
    };
    
    KEY.with(|storage| { 
        storage.borrow_mut().replace(key);
    });
}

#[query]
#[collect_metrics]
async fn get_secret(address: String, signature: String) -> String {
    let address = address[2..].to_string();
    let signature = signature[2..].to_string();

    let signature: Signature = match Signature::from_str(signature.as_str()) {
        Ok(signature) => signature,
        Err(error) => trap(&format!("Incorrect signature: {}", &error.to_string()))
    };

    let address_bytes: Vec<u8> = match hex::decode(&address) {
        Ok(bytes) => bytes,
        Err(error) => trap(&format!("Incorrect address: {}", &error.to_string()))
    };

    let address_bytes: [u8; 20] = match address_bytes.try_into() {
        Ok(bytes) => bytes,
        Err(_) => trap("Incorrect address lengh")
    };

    match signature.verify(MESSAGE, address_bytes) {
        Ok(_) => (),
        Err(message) => trap(&message.to_string())
    };

    let key = KEY.with(|storage| { 
        storage.borrow_mut().clone().unwrap()
    });

    let secret = address + &key;

    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    let result = hasher.finalize();

    hex::encode(result)
}

#[pre_upgrade]
fn pre_upgrade() {
    let key: Option<String> = KEY.with(|storage| { 
        storage.borrow_mut().clone()
    });
    let pre_upgrade_data = PersistedData {key};
    match storage::stable_save((pre_upgrade_data, 0)) {
        Ok(_) => (),
        Err(message) => trap(&format!("Failed to preupgrade: {}", message))
    }
}

#[post_upgrade]
fn post_upgrade() {
    match storage::stable_restore() {
        Ok(store) => {
            let (post_data, _): (PersistedData, i32) = store;
            if post_data.key.is_some() {
                KEY.with(|storage| { 
                    storage.borrow_mut().replace(post_data.key.unwrap());
                });
            }
        }
        Err(message) => trap(message.as_str())
    }
}

#[ic_cdk_macros::query(name = "getCanisterMetrics")]
pub async fn get_canister_metrics(parameters: canistergeek_ic_rust::api_type::GetMetricsParameters) -> Option<canistergeek_ic_rust::api_type::CanisterMetrics<'static>> {
    canistergeek_ic_rust::monitor::get_metrics(&parameters)
}

#[ic_cdk_macros::update(name = "collectCanisterMetrics")]
pub async fn collect_canister_metrics() -> () {
    canistergeek_ic_rust::monitor::collect_metrics();
}

#[ic_cdk_macros::query(name = "getCanisterLog")]
pub async fn get_canister_log(request: Option<canistergeek_ic_rust::api_type::CanisterLogRequest>) -> Option<canistergeek_ic_rust::api_type::CanisterLogResponse<'static>> {
    canistergeek_ic_rust::logger::get_canister_log(request)
}

fn main() {}
