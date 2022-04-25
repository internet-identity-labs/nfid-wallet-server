use ic_cdk::export::candid::{CandidType, Deserialize};

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ConfigurationRequest {
    pub identity_manager: String,
    pub token_ttl: Option<u64>,
}
