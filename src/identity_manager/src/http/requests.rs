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
    pub devices: Vec<Device>,
    pub personas: Vec<PersonaResponse>,
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

#[derive(Clone, Debug, CandidType, Deserialize, Copy)]
pub struct Configuration {
    pub lambda: Principal,
    pub token_ttl: u64
}
