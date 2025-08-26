use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use candid::export_service;
use candid::{CandidType, Principal};
use ic_cdk::api::call::CallResult;
use ic_cdk::api::management_canister::main::CanisterStatusResponse;
use ic_cdk::{call, caller, id, storage, trap};
use ic_cdk_macros::*;
use serde::{Deserialize, Serialize};

#[derive(CandidType, Deserialize, Clone, Serialize, Debug, Hash, PartialEq)]
pub struct Conf {
    pub im_canister: Option<String>,
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug, Hash, PartialEq, Eq)]
pub enum ICRC1State {
    Active,
    Inactive,
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug, Eq)]
pub struct ICRC1 {
    pub state: ICRC1State,
    pub ledger: String,
}

impl Hash for ICRC1 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ledger.hash(state);
    }
}

impl PartialEq for ICRC1 {
    fn eq(&self, other: &Self) -> bool {
        self.ledger == other.ledger
    }
}

thread_local! {
     static CONFIG: RefCell<Conf> = RefCell::new( Conf {
        im_canister: None
    });
    pub static ICRC_REGISTRY: RefCell<HashMap<String, HashSet<ICRC1>>> = RefCell::new(HashMap::default());
}

/// Persists the ICRC1 canister metadata for a specified user ledger ID principal.
#[update]
pub async fn store_icrc1_canister(ledger_id: String, state: ICRC1State) {
    let caller = get_root_id().await;
    ICRC_REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        let canister_id = ICRC1 { state, ledger: ledger_id.clone() };
        let canisters = registry.entry(caller).or_insert_with(HashSet::new);
        canisters.retain(|existing_canister| existing_canister.ledger != ledger_id);
        canisters.insert(canister_id);
    });
}

/// Removes the ICRC1 canister for a specified user ledger ID principal.
#[update]
pub async fn remove_icrc1_canister(ledger_id: String) {
    let caller = get_root_id().await;
    ICRC_REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        if let Some(canisters) = registry.get_mut(&caller) {
            canisters.retain(|canister| canister.ledger != ledger_id);
        }
    });
}

/// Invoked when the canister starts.
/// Initializes the application with `Conf` parameters and saves them to storage.
#[init]
pub async fn init(conf: Conf) {
    CONFIG.with(|c| c.replace(conf));
}

/// Returns all ICRC1 canisters persisted for the specified ledger ID principal.
#[query]
pub async fn get_canisters_by_root(root: String) -> Vec<ICRC1> {
    ICRC_REGISTRY.with(|registry| {
        let registry = registry.borrow();
        registry.get(&root).cloned().unwrap_or_default().into_iter().collect()
    })
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
struct Memory {
    registry: HashMap<String, HashSet<ICRC1>>,
    config: Conf,
}

/// Applies changes before the canister upgrade.
#[pre_upgrade]
pub fn stable_save() {
    let registry = ICRC_REGISTRY.with(|registry| {
        let registry = registry.borrow();
        registry.clone()
    });
    let config = CONFIG.with(|config| {
        let config = config.borrow();
        config.clone()
    });
    let mem = Memory { registry, config };
    storage::stable_save((mem,))
        .expect("Stable save exited unexpectedly: unable to save data to stable memory.");
}

/// Applies changes after the canister upgrade.
#[post_upgrade]
pub fn stable_restore() {
    let (mo,): (Memory,) = storage::stable_restore()
        .expect("Stable restore exited unexpectedly: unable to restore data from stable memory.");
    CONFIG.with(|mut config| {
        let mut config = config.borrow_mut();
        *config = mo.config.clone();
    });
    ICRC_REGISTRY.with(|mut registry| {
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

async fn get_root_id() -> String {
    match CONFIG.with(|c| c.borrow_mut().im_canister.clone()) {
        None => caller().to_text(), // Return caller for testing purposes when im_canister is None
        Some(canister) => {
            let princ = caller();
            let im_canister = Principal::from_text(canister)
                .expect("Unable to obtain Principal from im_canister.");

            match call(im_canister, "get_root_by_principal", (princ.to_text(), 0)).await {
                Ok((Some(root_id),)) => root_id,
                Ok((None,)) => trap("No root found for this principal"),
                Err((_, err)) => trap(&format!("Failed to request IM: {}", err)),
            }
        }
    }
}
