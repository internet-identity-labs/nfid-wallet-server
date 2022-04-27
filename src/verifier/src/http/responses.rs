use ic_cdk::export::candid::{CandidType, Deserialize};

pub fn to_success_response<T>(x: T) -> HttpResponse<T> {
    HttpResponse {
        data: Option::from(x),
        error: None,
        status_code: 200,
    }
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HttpResponse<T> {
    pub data: Option<T>,
    pub error: Option<String>,
    pub status_code: u16,
}
