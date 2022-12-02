use std::collections::HashMap;
use std::str::FromStr;

use canister_api_macros::{admin, collect_metrics};
use ethers_core::types::Signature;
use ethers_signers::{Wallet, LocalWallet};
use ethers_signers::Signer;
use ic_cdk::export::Principal;
use ic_cdk::export::candid::CandidType;
use ic_cdk::{trap, call, storage};
use serde::Deserialize;
use std::cell::RefCell;
use ic_cdk_macros::{init, update, pre_upgrade, post_upgrade};

use crate::repository::configuration_repo::{AdminRepo, ConfigurationRepo, Configuration, ControllersRepo};
use crate::service::ic_service::get_caller;

use hex; 
// use web3::signing::keccak256;
// use web3::signing::recover;
use ic_web3::signing::keccak256;
use secp256k1::{Message, PublicKey, Secp256k1, Signature};
// use ic_web3::signing::rec;

mod service;
mod repository;

thread_local! {
    static SECRET_STORAGE: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

#[init]
async fn init() -> () {
    AdminRepo::save(get_caller());
}

#[update]
#[admin]
#[collect_metrics]
async fn configure(sign_text: Option<String>) -> () {
    let configuration = Configuration {
        sign_text: sign_text.unwrap_or("You're authentification to NFID.".to_string()),
        whitelisted_canisters: None,
    };
    ConfigurationRepo::save(configuration);
}


pub(crate) fn verify_signature(message_hash: &[u8], public_key: &PublicKey, signature: &Signature) {
    // Verify the signature:
    let secp = Secp256k1::new();
    let message = Message::from_slice(message_hash).expect("32 bytes");
    assert!(secp.verify(&message, signature, public_key).is_ok());
}

#[update]
#[collect_metrics]
async fn secret_by_signature(signature: String) -> String {
    let sig: Signature = Signature::from_str("0x90069f397055f97fda932e22a15eaa80a8c4f827a0a777c1005a6e1d8dd5553f116421c402e4334d9aa649b0879c697ec0fa2b2143012632cb0572c7de86d07a1b").unwrap();
    let result = sig.recover("You're authentification to NFID.".to_string()).unwrap();
    let address = format!("{:?}", result);
    address.clone()
}

// async fn get_secret_by_signature(signature: String, sign_text: String) -> String {
//     let message = encode_message(sign_text);
//     let signature = hex::decode(&signature[2..]).unwrap();
//     let address = recover(&message[..32], &signature[..64], 0).unwrap();
//     let address = format!("{:?}", address);
//     let token = generate_token().await;

//     SECRET_STORAGE.with(|storage| {
//         let mut storage_mut = storage.borrow_mut();
//         return match storage_mut.get(&address) {
//             None => {
//                 storage_mut.insert(address.clone(), token.clone());
//                 token.clone()     
//              }
//             Some(secret) => { return secret.clone() }
//         };
//     })
// }

fn encode_message(message: String) -> [u8; 32] {
    keccak256(
        format!(
            "{}{}{}",
            "\x19Ethereum Signed Message:\n",
            message.len(),
            message
        )
        .as_bytes(),
    )
}
async fn generate_token() -> String {
    let token: Vec<u8> = match call(Principal::management_canister(), "raw_rand", ()).await {
        Ok((res, )) => res,
        Err((_, err)) => trap(&format!("failed to get salt: {}", err)),
    };

    let token: String = hex::encode(token);
    token
}

#[cfg(test)]
mod test {
    // use std::str::FromStr;
    // use super::*;

    // #[test]
    // fn should_return_secret_by_signature() {
    //     let message = "You're authentification to NFID.".to_string();
    //     let signature = "90069f397055f97fda932e22a15eaa80a8c4f827a0a777c1005a6e1d8dd5553f116421c402e4334d9aa649b0879c697".to_string();
    //     let expected_address = "0xdc75e8c3ae765d8947adbc6698a2403a6141d439".to_string();

    //     let arr: [u8;32] = [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1];
    //     let mut secp = Secp256k1::new();
    //     secp.seeded_randomize(&arr);


    //     let message_hash = encode_message(message);
    //     let message = Message::from_slice(&message_hash).expect("32 bytes");

    //     let signature: Signature = Signature::from_str(&signature).unwrap() ;
    //     let public_key = PublicKey::from_str(&"asdasd").unwrap();
    //     assert!(secp.verify(&message, &signature, &public_key).is_ok());

    use super::*;

    #[test]
     fn should_return_secret_by_signature() {
        let message = "You're authentification to NFID.".to_string();
        let signature = "0x90069f397055f97fda932e22a15eaa80a8c4f827a0a777c1005a6e1d8dd5553f116421c402e4334d9aa649b0879c697ec0fa2b2143012632cb0572c7de86d07a1b".to_string();
        let expected_address = "0xdc75e8c3ae765d8947adbc6698a2403a6141d439".to_string();

        let sig: Signature = Signature::from_str("0x90069f397055f97fda932e22a15eaa80a8c4f827a0a777c1005a6e1d8dd5553f116421c402e4334d9aa649b0879c697ec0fa2b2143012632cb0572c7de86d07a1b").unwrap();
        let result = sig.recover("You're authentification to NFID.".to_string()).unwrap();
        let address = format!("{:?}", result);
        print!("{:?}", address);


        // let secret1 = get_secret_by_signature(signature.clone(), message.clone()).await;
        // let secret2 = get_secret_by_signature(signature.clone(), message.clone()).await;
        
        // assert_eq!(secret1, secret2)
    }
}

#[pre_upgrade]
fn pre_upgrade() {
    let pre_upgrade_data = PreUpgradeData {
        admin: Some(AdminRepo::get()),
    };
    storage::stable_save((pre_upgrade_data, 0));
}

#[post_upgrade]
fn post_upgrade() {
    match storage::stable_restore() {
        Ok(store) => {
            let (post_data, _): (PostUpgradeData, i32) = store;
            if post_data.admin.is_some() {
                AdminRepo::save(post_data.admin.unwrap());
            }
        }
        Err(_) => ()
    }
}

#[derive(Clone, Debug, Deserialize, CandidType)]
pub struct PreUpgradeData {
    pub admin: Option<Principal>,
}

#[derive(Clone, Debug, Deserialize, CandidType)]
pub struct PostUpgradeData {
    pub admin: Option<Principal>,
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

fn main() {

}