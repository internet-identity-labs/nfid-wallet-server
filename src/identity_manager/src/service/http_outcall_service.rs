use std::convert::{TryInto, TryFrom};

use ic_cdk::{export::{candid::{CandidType, self}, Principal}, api::call::{RejectionCode, call_with_payment}};
use serde::{Deserialize, Serialize};
use candid::{
    parser::types::FuncMode,
    types::{Function, Serializer, Type}
};

#[ic_cdk_macros::query]
 pub fn transorm_response_no_headers(arg: TransformArgs) -> HttpResponse {
    HttpResponse {
        status: arg.response.status,
        headers: vec![],
        body: arg.response.body,
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct TransformFunc(pub candid::Func);

impl CandidType for TransformFunc {
    fn _ty() -> Type {
        Type::Func(Function {
            modes: vec![FuncMode::Query],
            args: vec![TransformArgs::ty()],
            rets: vec![HttpResponse::ty()],
        })
    }

    fn idl_serialize<S: Serializer>(&self, serializer: S) -> Result<(), S::Error> {
        serializer.serialize_function(self.0.principal.as_slice(), &self.0.method)
    }
}

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct TransformContext {
    pub function: TransformFunc,

    #[serde(with = "serde_bytes")]
    pub context: Vec<u8>,
}

impl TransformContext {
    pub fn new<T>(func: T, context: Vec<u8>) -> Self
    where
        T: Fn(TransformArgs) -> HttpResponse,
    {
        Self {
            function: TransformFunc(candid::Func {
                principal: id(),
                method: get_function_name(func).to_string(),
            }),
            context,
        }
    }
}

pub fn id() -> Principal {
    let len: u32 = unsafe { ic0::canister_self_size() as u32 };
    let mut bytes = vec![0u8; len as usize];
    unsafe {
        ic0::canister_self_copy(bytes.as_mut_ptr() as i32, 0, len as i32);
    }
    Principal::try_from(&bytes).unwrap()
}

fn get_function_name<F>(_: F) -> &'static str {
    let full_name = std::any::type_name::<F>();
    match full_name.rfind(':') {
        Some(index) => &full_name[index + 1..],
        None => full_name,
    }
}

#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct HttpResponse {
    pub status: candid::Nat,
    pub headers: Vec<HttpHeader>,
    pub body: Vec<u8>,
}

#[derive(CandidType, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransformArgs {
    pub response: HttpResponse,

    #[serde(with = "serde_bytes")]
    pub context: Vec<u8>,
}

/// Argument type of [http_request].
#[derive(CandidType, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct CanisterHttpRequestArgument {
    pub url: String,
    pub max_response_bytes: Option<u64>,
    pub method: HttpMethod,
    pub headers: Vec<HttpHeader>,
    pub body: Option<Vec<u8>>,
    pub transform: Option<TransformContext>,
}

pub type CallResult<R> = Result<R, (RejectionCode, String)>;

#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct HttpHeader {
    pub name: String,
    pub value: String,
}

#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy,
)]
pub enum HttpMethod {
    #[serde(rename = "get")]
    GET,
    #[serde(rename = "post")]
    POST,
    #[serde(rename = "head")]
    HEAD,
}

#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct HttpResponseHttpOutcall {
    pub status: candid::Nat,
    pub headers: Vec<HttpHeader>,
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