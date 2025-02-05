use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct PrincipalEmailRequest {
    pub principal_id: String,
    pub email: String,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct AccountRequest {
    pub anchor: u64,
    pub wallet: Option<WalletVariant>,
    pub access_point: Option<AccessPointRequest>,
    pub email: Option<String>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct AccountUpdateRequest {
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct TokenRequest {
    pub phone_number_encrypted: String,
    pub phone_number_hash: String,
    pub token: String,
    pub principal_id: String,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ValidatePhoneRequest {
    pub phone_number_hash: String,
    pub principal_id: String,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct AccountResponse {
    pub principal_id: String,
    pub name: Option<String>,
    pub phone_number: Option<String>,
    pub personas: Vec<PersonaResponse>,
    pub access_points: Vec<AccessPointResponse>,
    pub anchor: u64,
    pub wallet: WalletVariant,
    pub is2fa_enabled: bool,
    pub email: Option<String>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum CredentialVariant {
    #[serde(rename = "phone_number")]
    PhoneNumber(PhoneNumberCredential),
}

#[derive(Clone, Copy, Debug, CandidType, Deserialize, PartialEq, Serialize)]
pub enum WalletVariant {
    #[serde(rename = "NFID")]
    NFID,
    #[serde(rename = "II")]
    InternetIdentity,
}

#[derive(Clone, Copy, Debug, CandidType, Deserialize, PartialEq, Eq, Serialize, Hash)]
pub enum DeviceType {
    #[serde(rename = "Email")]
    Email,
    #[serde(rename = "Passkey")]
    Passkey,
    #[serde(rename = "Recovery")]
    Recovery,
    #[serde(rename = "Unknown")]
    Unknown,
    #[serde(rename = "Password")]
    Password,
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
    pub lambda_url: Option<String>,
    pub lambda: Option<Principal>,
    pub token_ttl: Option<u64>,
    pub token_refresh_ttl: Option<u64>,
    pub whitelisted_phone_numbers: Option<Vec<String>>,
    pub heartbeat: Option<u32>,
    pub backup_canister_id: Option<String>,
    pub ii_canister_id: Option<Principal>,
    pub whitelisted_canisters: Option<Vec<Principal>>,
    pub env: Option<String>,
    pub git_branch: Option<String>,
    pub commit_hash: Option<String>,
    pub operator: Option<Principal>,
    pub account_creation_paused: Option<bool>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ConfigurationResponse {
    pub lambda_url: Option<String>,
    pub lambda: Option<Principal>,
    pub token_ttl: Option<u64>,
    pub token_refresh_ttl: Option<u64>,
    pub whitelisted_phone_numbers: Option<Vec<String>>,
    pub heartbeat: Option<u32>,
    pub backup_canister_id: Option<String>,
    pub ii_canister_id: Option<Principal>,
    pub whitelisted_canisters: Option<Vec<Principal>>,
    pub env: Option<String>,
    pub git_branch: Option<String>,
    pub commit_hash: Option<String>,
    pub operator: Option<Principal>,
    pub account_creation_paused: Option<bool>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct AccessPointResponse {
    pub principal_id: String,
    pub credential_id: Option<String>,
    pub icon: String,
    pub device: String,
    pub browser: String,
    pub last_used: u64,
    pub device_type: DeviceType,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct AccessPointRequest {
    pub pub_key: String,
    pub icon: String,
    pub device: String,
    pub browser: String,
    pub device_type: DeviceType,
    pub credential_id: Option<String>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct AccessPointRemoveRequest {
    pub pub_key: String,
}
