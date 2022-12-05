use ic_cdk::export::candid::CandidType;
use serde::Deserialize;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HttpResponse<T> {
    pub data: Option<T>,
    pub error: Option<String>,
    pub status_code: u16,
}

pub fn get_success<T>(data: T) -> HttpResponse<T> {
    HttpResponse {
        data: Option::from(data),
        error: None,
        status_code: 200,
    }
}

pub fn get_error(error: String, status_code: u16) -> HttpResponse<String> {
    HttpResponse {
        data: None,
        error: Some(error),
        status_code,
    }
}