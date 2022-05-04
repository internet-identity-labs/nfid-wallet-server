use std::collections::{HashSet};

use ic_cdk::{storage, trap};
use ic_cdk_macros::*;
use ic_cdk::export::Principal;
use canister_api_macros::{collect_metrics};

pub type ProofOfAttendanceProtocol = HashSet<Principal>;


#[query]
async fn ping() -> () {}

#[query]
async fn has_poap() -> bool {
    let princ = ic_cdk::api::caller();
    let index = storage::get_mut::<ProofOfAttendanceProtocol>();
    return index.contains(&princ);
}

#[update]
#[collect_metrics]
async fn increment_poap() {
    let princ = ic_cdk::api::caller();
    let index = storage::get_mut::<ProofOfAttendanceProtocol>();
    index.insert(princ);
}

#[pre_upgrade]
async fn pre_upgrade() {
    storage::stable_save((storage::get_mut::<ProofOfAttendanceProtocol>(), 0));
}

#[post_upgrade]
fn post_upgrade() {
    let poap: (HashSet<Principal>, i32) = storage::stable_restore().unwrap();
    let index = storage::get_mut::<ProofOfAttendanceProtocol>();
    for p in poap.0.iter() {
        index.insert(p.to_owned());
    }
}

#[ic_cdk_macros::query(name = "getCanisterMetrics")]
pub async fn get_canister_metrics(parameters: canistergeek_ic_rust::api_type::GetMetricsParameters) -> Option<canistergeek_ic_rust::api_type::CanisterMetrics<'static>> {
    canistergeek_ic_rust::monitor::get_metrics(&parameters)
}

#[ic_cdk_macros::update(name = "collectCanisterMetrics")]
pub async fn collect_canister_metrics() -> () {
    canistergeek_ic_rust::monitor::collect_metrics();
}

#[ic_cdk_macros::query(name = "getCanisterLog")]
pub async fn get_canister_log(request: Option<canistergeek_ic_rust::api_type::CanisterLogRequest>) -> Option<canistergeek_ic_rust::api_type::CanisterLogResponse<'static>> {
    canistergeek_ic_rust::logger::get_canister_log(request)
}

fn main() {}
