use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;

use crate::repository::repo::AccessPoint;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HTTPAccountRequest {
    pub name: String,
    pub phone_number: String,
    pub token: String,
    pub anchor: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HTTPAccountUpdateRequest {
    pub name: Option<String>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HTTPVerifyPhoneNumberRequest {
    pub phone_number: String,
    pub token: String,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct AccountResponse {
    pub principal_id: String,
    pub name: String,
    pub phone_number: String,
    pub access_points: Vec<AccessPoint>,
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
pub struct Configuration {
    pub lambda: Principal,
    pub token_ttl: u64,
    pub key: [u8; 32],
    pub whitelisted_phone_numbers: Option<Vec<String>>
}
