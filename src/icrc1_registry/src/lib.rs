use std::cell::RefCell;
use std::collections::HashMap;

use candid::{CandidType, Principal};
use candid::export_service;
use ic_cdk::{call, caller, id, storage, trap};
use ic_cdk::api::call::CallResult;
use ic_cdk::api::management_canister::main::CanisterStatusResponse;
use ic_cdk_macros::*;
use serde::{Deserialize, Serialize};

thread_local! {
    pub static ICRC_REGISTRY: RefCell<HashMap<Principal, Vec<String>>> = RefCell::new(HashMap::default());
}

#[update]
pub async fn add_icrc1_canister(canister_id: String) {
    let caller = caller();
    ICRC_REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        let canisters = registry.entry(caller).or_insert_with(Vec::new);
        canisters.push(canister_id);
    });
}

#[query]
pub async fn get_canisters() -> Vec<String> {
    let caller = caller();
    ICRC_REGISTRY.with(|registry| {
        let registry = registry.borrow();
        registry.get(&caller).cloned().unwrap_or_default()
    })
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
struct Memory {
    registry: HashMap<Principal, Vec<String>>,
}

#[pre_upgrade]
pub fn stable_save() {
    let registry = ICRC_REGISTRY.with(|registry| {
        let registry = registry.borrow();
        registry.clone()
    });
    let mem = Memory {
        registry
    };
    storage::stable_save((mem, )).unwrap();
}

#[post_upgrade]
pub fn stable_restore() {
    let (mo, ): (Memory, ) = storage::stable_restore().unwrap();
    ICRC_REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        *registry = mo.registry;
    });
}

#[test]
fn sub_account_test() {}
export_service!();

#[ic_cdk_macros::query(name = "__get_candid_interface")]
fn export_candid() -> String {
    __export_service()
}