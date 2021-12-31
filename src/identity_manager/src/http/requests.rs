use ic_cdk::export::candid::{CandidType, Deserialize};
use crate::repository::repo::Device;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HTTPAccountRequest {
    pub name: String,
    pub phone_number: String,
    pub email: String,
    pub token: String,
    pub is_seed_phrase_copied: bool,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HTTPAccountUpdateRequest {
    pub name: Option<String>,
    pub phone_number: Option<String>,
    pub email: Option<String>,
    pub is_seed_phrase_copied: Option<bool>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HTTPPersonaUpdateRequest {
    pub anchor: Option<String>,
    pub application: Option<String>, //todo temp
    pub principal_id: String,
    pub application_user_name: Option<String>,
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
    pub email: String,
    pub devices: Vec<Device>,
    pub personas: Vec<PersonaResponse>,
    pub is_seed_phrase_copied: bool,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct PersonaResponse {
    pub anchor: Option<String>,
    pub principal_id: String,
    pub application_user_name: Option<String>, //todo temp
    pub application: Option<String>, //todo temp
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct PersonaRequest {
    pub anchor: Option<String>,
    pub principal_id: String,
    pub principal_id_origin: String,
    pub application_user_name: Option<String>, //todo temp
    pub application: Option<String>, //todo temp
}
