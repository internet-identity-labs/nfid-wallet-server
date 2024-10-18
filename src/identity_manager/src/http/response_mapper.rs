use candid::{CandidType, Deserialize};

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

pub trait ErrorResponse<T> {
    fn error(status_code: u16, text: &str) -> HttpResponse<T>;
}

pub trait DataResponse<T> {
    fn data(status_code: u16, data: T) -> HttpResponse<T>;
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

pub fn to_success_response<T>(x: T) -> HttpResponse<T> {
    HttpResponse {
        data: Option::from(x),
        error: None,
        status_code: 200,
    }
}

impl<T> ErrorResponse<T> for HttpResponse<T> {
    fn error(status_code: u16, text: &str) -> HttpResponse<T> {
        HttpResponse {
            data: None,
            error: Some(String::from(text)),
            status_code,
        }
    }
}

impl<T> DataResponse<T> for HttpResponse<T> {
    fn data(status_code: u16, data: T) -> HttpResponse<T> {
        HttpResponse {
            data: Some(data),
            error: None,
            status_code,
        }
    }
}
