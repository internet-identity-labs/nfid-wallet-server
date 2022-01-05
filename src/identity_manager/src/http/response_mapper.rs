use ic_cdk::export::candid::{CandidType, Deserialize};

type Error = String;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HttpResponse<T> {
    pub data: Option<T>,
    pub error: Option<Error>,
    pub status_code: u16,
}

pub fn to_error_response<T>(x: &str) -> HttpResponse<T> {
    HttpResponse {
        data: None,
        error: Some(String::from(x)),
        status_code: 404,
    }
}

pub fn unauthorized() -> HttpResponse<bool> {
    HttpResponse {
        data: None,
        error: Some(Error::from("Unauthorized")),
        status_code: 404,
    }
}

pub fn to_success_response<T>(x: T) -> HttpResponse<T> {
    HttpResponse {
        data: Option::from(x),
        error: None,
        status_code: 200,
    }
}

