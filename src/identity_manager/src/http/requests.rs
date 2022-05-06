use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use crate::repository::access_point_repo::AccessPoint;

use serde_bytes::{ByteBuf};


#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct AccountRequest {
    pub anchor: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct AccountUpdateRequest {
    pub name: Option<String>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct TokenRequest {
    pub phone_number_encrypted: String,
    pub phone_number_hash: String,
    pub token: String,
    pub principal_id: String
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ValidatePhoneRequest {
    pub phone_number_hash: String,
    pub principal_id: String
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct AccountResponse {
    pub principal_id: String,
    pub name: Option<String>,
    pub phone_number: Option<String>,
    pub personas: Vec<PersonaResponse>,
    pub access_points: Vec<AccessPointResponse>,
    pub anchor: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum CredentialVariant {
    #[serde(rename = "phone_number")]
    PhoneNumber(PhoneNumberCredential),
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct PhoneNumberCredential {
    pub phone_number: String,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct PersonaResponse {
    pub domain: String,
    pub persona_id: String,
    pub persona_name: String,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct PersonaRequest {
    pub domain: String,
    pub persona_name: String,
    pub persona_id: String,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ConfigurationRequest {
    pub lambda: Principal,
    pub token_ttl: u64,
    pub token_refresh_ttl: u64,
    pub whitelisted_phone_numbers: Option<Vec<String>>,
    pub heartbeat: Option<u32>,
    pub backup_canister_id: Option<String>,
    pub ii_canister_id: Option<Principal>,
    pub whitelisted_canisters: Option<Vec<Principal>>,
    pub env: Option<String>,
    pub git_branch: Option<String>,
    pub commit_hash: Option<String>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ConfigurationResponse {
    pub lambda: Principal,
    pub token_ttl: u64,
    pub token_refresh_ttl: u64,
    pub whitelisted_phone_numbers: Option<Vec<String>>,
    pub heartbeat: Option<u32>,
    pub backup_canister_id: Option<String>,
    pub ii_canister_id: Option<Principal>,
    pub whitelisted_canisters: Option<Vec<Principal>>,
    pub env: Option<String>,
    pub git_branch: Option<String>,
    pub commit_hash: Option<String>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct AccessPointResponse {
    pub principal_id: String,
    pub icon: String,
    pub device: String,
    pub browser: String,
    pub last_used: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct AccessPointRequest {
    pub pub_key: ByteBuf,
    pub icon: String,
    pub device: String,
    pub browser: String,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct AccessPointRemoveRequest {
    pub pub_key: ByteBuf,
}