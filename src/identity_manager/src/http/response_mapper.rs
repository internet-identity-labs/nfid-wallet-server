use ic_cdk::export::candid::{CandidType, Deserialize};

type Error = String;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HttpResponse<T> {
    pub data: Option<T>,
    pub error: Option<Error>,
    pub status_code: u16,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Response {
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

pub fn error_response(code: u16, text: &str) -> Response {
    Response {
        error: Some(String::from(text)),
        status_code: code,
    }
}

pub fn response(code: u16) -> Response {
    Response {
        error: None,
        status_code: code,
    }
}

pub fn unauthorized() -> Response {
    Response {
        error: Some(Error::from("Unauthorized")),
        status_code: 404,
    }
}

pub fn too_many_requests() -> HttpResponse<bool> {
    HttpResponse {
        data: None,
        error: Some(Error::from("Too many requests")),
        status_code: 429,
    }
}

pub fn to_success_response<T>(x: T) -> HttpResponse<T> {
    HttpResponse {
        data: Option::from(x),
        error: None,
        status_code: 200,
    }
}

