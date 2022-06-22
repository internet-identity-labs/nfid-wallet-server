use ic_cdk::{call, trap};
use ic_cdk::export::Principal;
use crate::ConfigurationRepo;
use ic_cdk::export::candid::{CandidType, Deserialize};
use serde_bytes::ByteBuf;

type CredentialId = ByteBuf;
type PublicKey = ByteBuf;
type DeviceKey = PublicKey;

#[derive(Clone, Debug, CandidType, Deserialize)]
enum Purpose {
    #[serde(rename = "recovery")]
    Recovery,
    #[serde(rename = "authentication")]
    Authentication,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct DeviceData {
    pubkey: DeviceKey,
    alias: String,
    credential_id: Option<CredentialId>,
    purpose: Purpose,
    key_type: KeyType,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
enum KeyType {
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "platform")]
    Platform,
    #[serde(rename = "cross_platform")]
    CrossPlatform,
    #[serde(rename = "seed_phrase")]
    SeedPhrase,
}

#[cfg(test)]
pub fn get_caller() -> Principal {
    Principal::anonymous()
}

#[cfg(not(test))]
pub fn get_caller() -> Principal {
    ic_cdk::api::caller()
}

#[cfg(test)]
pub fn get_time() -> u64 {
    123456789
}

#[cfg(not(test))]
pub fn get_time() -> u64 {
    ic_cdk::api::time()
}

#[cfg(test)]
pub fn is_anonymous(_princ: String) -> bool {
    false
}

#[cfg(not(test))]
pub fn is_anonymous(princ: String) -> bool {
    princ.len() < 10
}

pub async fn trap_if_not_authenticated(anchor: u64, principal: Principal) {
    if ConfigurationRepo::get().env.as_ref().is_some()
        && ConfigurationRepo::get().env.as_ref().unwrap().eq(&"test".to_string()) {
        return;
    }

    let ii_canister = ConfigurationRepo::get().ii_canister_id;

    //TODO update when possible to query call
    let res: Vec<DeviceData> = match call(ii_canister, "lookup", (anchor.clone(), 0)).await {
        Ok((res, )) => res,
        Err((_, err)) => trap(&format!("failed to request II: {}", err)),
    };

    verify(principal, res.iter().map(|e| &e.pubkey));
}

fn verify<'a>(princ: Principal, public_keys: impl Iterator<Item=&'a PublicKey>) {
    for pk in public_keys {
        if princ.clone() == Principal::self_authenticating(pk) {
            return;
        }
    }
    ic_cdk::trap(&format!("{} could not be authenticated.", princ))
}


