use std::collections::{HashSet};

use ic_cdk::{storage, trap};
use ic_cdk_macros::*;
use ic_cdk::export::Principal;

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

fn main() {}
