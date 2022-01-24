use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;

use crate::repository::repo::Device;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HTTPAccountRequest {
    pub name: String,
    pub phone_number: String,
    pub token: String,
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
    pub devices: Vec<Device>,
    pub personas: Vec<PersonaVariant>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum PersonaVariant {
    #[serde(rename = "nfid_persona")]
    NfidPersona(PersonaNFIDResponse),
    #[serde(rename = "ii_persona")]
    IiPersona(PersonaIIResponse),
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct PersonaIIResponse {
    pub anchor: String,
    pub domain: String,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct PersonaNFIDResponse {
    pub domain: String,
    pub persona_id: String,
}

#[derive(Clone, Debug, CandidType, Deserialize, Copy)]
pub struct Configuration {
    pub lambda: Principal,
    pub token_ttl: u64,
    pub key: [u8; 32]
}
