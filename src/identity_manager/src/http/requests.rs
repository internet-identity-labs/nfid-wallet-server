use ic_cdk::export::candid::{CandidType, Deserialize};

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HTTPAccountRequest {
    pub name: String,
    pub phone_number: String,
    pub email: String,
    pub token: String,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HTTPAccountUpdateRequest {
    pub name: Option<String>,
    pub phone_number: Option<String>,
    pub email: Option<String>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HTTPPersonaUpdateRequest {
    pub name: Option<String>,
    pub is_root: Option<bool>,
    pub is_seed_phrase_copied: Option<bool>,
    pub is_ii_anchor: Option<bool>,
    pub anchor: Option<String>,
    pub principal_id: String,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HTTPVerifyPhoneNumberRequest {
    pub phone_number: String,
    pub token: String,
}
