use crate::ConfigurationRepo;
use candid::{CandidType, Deserialize, Nat, Principal};
use ic_cdk::api::call::CallResult;
use ic_cdk::{call, id, trap};
use serde::Serialize;
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

#[derive(Clone, Debug, CandidType, Deserialize, PartialEq)]
pub enum KeyType {
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "platform")]
    Platform,
    #[serde(rename = "cross_platform")]
    CrossPlatform,
    #[serde(rename = "seed_phrase")]
    SeedPhrase,
    #[serde(rename = "browser_storage_key")]
    BrowserStorageKey,
}

#[derive(Serialize, Deserialize, CandidType, Clone, PartialEq, Eq, Debug)]
pub enum CanisterStatus {
    #[serde(rename = "running")]
    running,
    #[serde(rename = "stopping")]
    stopping,
    #[serde(rename = "stopped")]
    stopped,
}

#[derive(Deserialize, CandidType, Clone, PartialEq, Eq, Debug)]
pub struct DefiniteCanisterSettings {
    controllers: Vec<Principal>,
    compute_allocation: Nat,
    memory_allocation: Nat,
    freezing_threshold: Nat,
}

#[derive(Deserialize, CandidType, Clone, PartialEq, Eq, Debug)]
pub struct CanisterStatusResponse {
    status: CanisterStatus,
    settings: DefiniteCanisterSettings,
    module_hash: Option<Vec<u8>>,
    memory_size: Nat,
    cycles: Nat,
    freezing_threshold: Nat,
}

#[derive(CandidType, Debug, Clone, Deserialize)]
pub struct CanisterIdRequest {
    #[serde(rename = "canister_id")]
    pub canister_id: Principal,
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

pub async fn get_controllers() -> Vec<Principal> {
    let res: CallResult<(CanisterStatusResponse,)> = call(
        Principal::management_canister(),
        "canister_status",
        (CanisterIdRequest { canister_id: id() },),
    )
    .await;


    return res
        .expect("Get controllers function exited unexpectedly: inter-canister call to management canister for canister_status returned an empty result.")
        .0.settings.controllers;
}

pub async fn trap_if_not_authenticated(anchor: u64, principal: Principal) -> Vec<DeviceData> {
    if ConfigurationRepo::get().env.is_some()
        && ConfigurationRepo::get()
            .env
            .as_ref()
            .expect("Failed to extract the env field from configuration.")
            .eq(&"test".to_string())
    {
        return Vec::default();
    }

    let ii_canister = ConfigurationRepo::get().ii_canister_id;

    //TODO update when possible to query call
    let res: Vec<DeviceData> = match call(ii_canister, "lookup", (anchor.clone(), 0)).await {
        Ok((res,)) => res,
        Err((_, err)) => trap(&format!("failed to request II: {}", err)),
    };

    verify(principal, res.iter().map(|e| &e.pubkey));
    res
}

fn verify<'a>(princ: Principal, public_keys: impl Iterator<Item = &'a PublicKey>) {
    for pk in public_keys {
        if princ.clone() == Principal::self_authenticating(pk) {
            return;
        }
    }
    ic_cdk::trap(&format!("{} could not be authenticated.", princ))
}

pub async fn get_device_data_vec(anchor: u64) -> Vec<DeviceData> {
    let ii_canister = ConfigurationRepo::get().ii_canister_id;
    let res: Vec<DeviceData> = match call(ii_canister, "lookup", (anchor.clone(), 0)).await {
        Ok((res,)) => res,
        Err((_, err)) => trap(&format!("failed to request II: {}", err)),
    };
    res
}
