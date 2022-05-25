use ic_cdk::{call, trap};
use ic_cdk::export::Principal;
use crate::ConfigurationRepo;
use ic_cdk::export::candid::{CandidType, Deserialize};
use serde_bytes::ByteBuf;

pub type CredentialId = ByteBuf;
pub type PublicKey = ByteBuf;
pub type DeviceKey = PublicKey;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum Purpose {
    #[serde(rename = "recovery")]
    Recovery,
    #[serde(rename = "authentication")]
    Authentication,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct DeviceData {
    pub pubkey: DeviceKey,
    pub alias: String,
    pub credential_id: Option<CredentialId>,
    pub purpose: Purpose,
    pub key_type: KeyType,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum KeyType {
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

pub async fn lookup(anchor: u64) -> Vec<DeviceData> {
    if is_test_env() {
        return Vec::new();
    }

    let ii_canister = ConfigurationRepo::get().ii_canister_id.unwrap();

    //TODO update when possible to query call
    match call(ii_canister, "lookup", (anchor.clone(), 0)).await {
        Ok((res, )) => res,
        Err((_, err)) => Vec::new()
    }
}

pub async fn trap_if_not_authenticated(anchor: u64, principal: Principal) {
    if is_test_env() {
        return;
    }

    let devices: Vec<DeviceData> = lookup(anchor.clone()).await;
    verify(principal, devices);
}

pub fn trap_if_not_authenticated_by_devices(devices: Vec<DeviceData>, principal: Principal) {
    if is_test_env() {
        return;
    }

    verify(principal, devices);
}

pub fn verify(principal: Principal, devices: Vec<DeviceData>) {
    for device in devices {
        if principal.clone() == Principal::self_authenticating(&device.pubkey) {
            return;
        }
    }
    trap(&format!("{} could not be authenticated.", principal))
}

fn is_test_env() -> bool {
    ConfigurationRepo::get().env.as_ref().is_some()
        && ConfigurationRepo::get().env.as_ref().unwrap().eq(&"test".to_string())
}


