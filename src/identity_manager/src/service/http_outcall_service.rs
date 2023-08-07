use std::convert::TryInto;

use ic_cdk::{export::{candid::{CandidType, self, Nat}, Principal}, api::call::{RejectionCode, call_with_payment}};
use serde::{Deserialize, Serialize};

/// Argument type of [http_request].
#[derive(CandidType, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct CanisterHttpRequestArgument {
    /// The requested URL.
    pub url: String,
    /// The maximal size of the response in bytes. If None, 2MiB will be the limit.
    pub max_response_bytes: Option<u64>,
    /// The method of HTTP request.
    pub method: HttpMethod,
    /// List of HTTP request headers and their corresponding values.
    pub headers: Vec<HttpHeader>,
    /// Optionally provide request body.
    pub body: Option<Vec<u8>>,
}

pub type CallResult<R> = Result<R, (RejectionCode, String)>;

#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct HttpHeader {
    /// Name
    pub name: String,
    /// Value
    pub value: String,
}

#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy,
)]
pub enum HttpMethod {
    /// GET
    #[serde(rename = "get")]
    GET,
    /// POST
    #[serde(rename = "post")]
    POST,
    /// HEAD
    #[serde(rename = "head")]
    HEAD,
}

#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct HttpResponseHttpOutcall {
    /// The response status (e.g., 200, 404).
    pub status: candid::Nat,
    /// List of HTTP response headers and their corresponding values.
    pub headers: Vec<HttpHeader>,
    /// The response’s body.
    pub body: Vec<u8>,
}

pub async fn http_request(arg: CanisterHttpRequestArgument) -> CallResult<(HttpResponseHttpOutcall,)> {
    let cycles = http_request_required_cycles(&arg);
    call_with_payment(
        Principal::management_canister(),
        "http_request",
        (arg,),
        cycles.try_into().unwrap(),
    )
    .await
}

fn http_request_required_cycles(arg: &CanisterHttpRequestArgument) -> u128 {
    let max_response_bytes = match arg.max_response_bytes {
        Some(ref n) => *n as u128,
        None => 2 * 1024 * 1024u128, // default 2MiB
    };
    let arg_raw = candid::utils::encode_args((arg,)).expect("Failed to encode arguments.");
    // The coefficients can be found in [this page](https://internetcomputer.org/docs/current/developer-docs/production/computation-and-storage-costs).
    // 12 is "http_request".len().
    400_000_000u128 + 100_000u128 * (arg_raw.len() as u128 + 12 + max_response_bytes)
}