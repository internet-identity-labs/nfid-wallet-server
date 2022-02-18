use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;


#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct AccountRequest {
    pub name: String,
    pub anchor: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct AccountUpdateRequest {
    pub name: Option<String>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct TokenRequest {
    pub phone_number: String,
    pub token: String,
    pub principal_id: String
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ValidatePhoneRequest {
    pub phone_number: String,
    pub principal_id: String
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct AccountResponse {
    pub principal_id: String,
    pub name: String,
    pub phone_number: Option<String>,
    pub personas: Vec<PersonaVariant>,
    pub anchor: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum PersonaVariant {
    #[serde(rename = "nfid_persona")]
    NfidPersona(PersonaNFID),
    #[serde(rename = "ii_persona")]
    IiPersona(PersonaII),
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct PersonaII {
    pub anchor: u64,
    pub domain: String,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct PersonaNFID {
    pub domain: String,
    pub persona_id: String,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ConfigurationRequest {
    pub lambda: Principal,
    pub token_ttl: u64,
    pub token_refresh_ttl: u64,
    pub key: [u8; 32],
    pub whitelisted_phone_numbers: Option<Vec<String>>
}
