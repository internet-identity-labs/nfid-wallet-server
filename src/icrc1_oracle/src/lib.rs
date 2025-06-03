use core::hash::Hash;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use candid::{candid_method, export_service};
use candid::{CandidType, Nat, Principal};
use ic_cdk::api::call::CallResult;
use ic_cdk::{call, caller, id, storage, trap};
use ic_cdk_macros::*;
use serde::{Deserialize, Serialize};

use crate::signer::AllowSigningError;

mod signer;
mod timer_service;

#[derive(CandidType, Deserialize, Clone, Debug, Hash, PartialEq, Eq, Serialize)]
pub enum Category {
    Spam,
    Known,
    Native,
    ChainFusion,
    ChainFusionTestnet,
    Community,
    Sns,
}

#[derive(CandidType, Debug, Clone, Deserialize)]
pub struct CanisterIdRequest {
    #[serde(rename = "canister_id")]
    pub canister_id: Principal,
}

#[derive(Serialize, Deserialize, CandidType, Clone, PartialEq, Eq, Debug)]
pub enum CanisterStatus {
    #[serde(rename = "running")]
    running,
    #[serde(rename = "stopping")]
    stopping,
    #[serde(rename = "stopped")]
    stopped,
}

#[derive(Deserialize, CandidType, Clone, PartialEq, Eq, Debug)]
pub struct DefiniteCanisterSettings {
    controllers: Vec<Principal>,
    compute_allocation: Nat,
    memory_allocation: Nat,
    freezing_threshold: Nat,
}

#[derive(Deserialize, CandidType, Clone, PartialEq, Eq, Debug)]
pub struct CanisterStatusResponse {
    status: CanisterStatus,
    settings: DefiniteCanisterSettings,
    module_hash: Option<Vec<u8>>,
    memory_size: Nat,
    cycles: Nat,
    freezing_threshold: Nat,
}

#[derive(CandidType, Deserialize, Clone, Debug, Hash, Serialize, PartialEq)]
pub struct Conf {
    pub im_canister: Option<Principal>,
    pub operator: Option<Principal>,
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug, Eq)]
pub struct ICRC1 {
    pub index: Option<String>,
    pub ledger: String,
    pub name: String,
    pub logo: Option<String>,
    pub symbol: String,
    pub category: Category,
    pub decimals: u8,
    pub fee: Nat,
    pub root_canister_id: Option<String>,
    pub date_added: u64,
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

#[derive(CandidType, Deserialize, Clone, Serialize, Debug, Eq)]
pub struct NeuronData {
    pub name: String,
    pub ledger: String,
    pub neuron_id: String,
    pub date_added: u64,
}

impl Hash for NeuronData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ledger.hash(state);
    }
}

impl PartialEq for NeuronData {
    fn eq(&self, other: &Self) -> bool {
        self.ledger == other.ledger
    }
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug, Hash, PartialEq, Eq)]
pub struct ICRC1Request {
    pub index: Option<String>,
    pub ledger: String,
    pub name: String,
    pub logo: Option<String>,
    pub symbol: String,
    pub decimals: u8,
    pub fee: Nat,
}

thread_local! {
     static CONFIG: RefCell<Conf> = RefCell::new( Conf {
        im_canister: None,
        operator: None
    });
    pub static ICRC_REGISTRY: RefCell<HashSet<ICRC1>> = RefCell::new(HashSet::default());
    pub static NEURON_REGISTRY: RefCell<HashSet<NeuronData>> = RefCell::new(HashSet::default());
}

/// Persists a single ICRC1 canister's metadata into the canister's storage.
#[update]
pub async fn store_icrc1_canister(request: ICRC1Request) {
    get_root_id().await;
    Principal::from_text(request.ledger.clone()).unwrap_or_else(|_| {
        trap("Invalid ledger principal");
    });
    if request.index.is_some() {
        Principal::from_text(
            request
                .index
                .clone()
                .expect("The request.index failed after existence check."),
        )
        .unwrap_or_else(|_| {
            trap("Invalid index principal");
        });
    }
    ICRC_REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        let canister_id = ICRC1 {
            index: request.index,
            ledger: request.ledger,
            name: request.name,
            logo: request.logo,
            symbol: request.symbol,
            category: Category::Community,
            decimals: request.decimals,
            fee: request.fee,
            root_canister_id: None,
            date_added: ic_cdk::api::time(),
        };
        registry.replace(canister_id);
    });
}

/// Invoked when the canister starts.
/// Initializes the application with `Conf` parameters and saves them to storage.
#[init]
pub async fn init(conf: Option<Conf>) {
    match conf {
        Some(conf) => {
            CONFIG.with(|storage| {
                storage.replace(conf);
            });
        }
        _ => {}
    };
}

/// Returns all persisted ICRC1 canisters.
#[query]
pub async fn get_all_icrc1_canisters() -> HashSet<ICRC1> {
    ICRC_REGISTRY.with(|registry| {
        let registry = registry.borrow();
        registry.clone()
    })
}

/// Returns amount of all persisted ICRC1 canisters.
#[query]
pub async fn count_icrc1_canisters() -> u64 {
    ICRC_REGISTRY.with(|registry| {
        let registry = registry.borrow();
        registry.len() as u64
    })
}

/// Retirns paginated response
#[query]
pub async fn get_icrc1_paginated(offset: u64, limit: u64) -> Vec<ICRC1> {
    ICRC_REGISTRY.with(|registry| {
        let registry = registry.borrow();
        registry
            .iter()
            .skip(offset as usize)
            .take(limit as usize)
            .cloned()
            .collect()
    })
}

/// Replaces the existing ICRC1 canisters with the provided list.
#[update]
pub async fn replace_icrc1_canisters(icrc1: Vec<ICRC1>) {
    trap_if_not_authenticated_admin();
    ICRC_REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        for canister in icrc1 {
            registry.replace(canister);
        }
    })
}

/// Persists an array of ICRC1 canisters under a specified category.
#[update]
pub async fn store_new_icrc1_canisters(icrc1: Vec<ICRC1>) {
    trap_if_not_authenticated_admin();
    ICRC_REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        for mut canister in icrc1 {
            let existent_canister = registry.get(&canister);
            //if canister exists - update metadata. Sometimes SNS logo can be updated silently
            if existent_canister.is_some() {
                canister.index = existent_canister.unwrap().index.clone();
                canister.date_added = existent_canister.unwrap().date_added;
                registry.replace(canister);
            } else {
                registry.insert(canister);
            }
        }
    })
}

/// Removes an ICRC1 canister by its ledger principal.
#[update]
pub async fn remove_icrc1_canister(ledger: String) {
    trap_if_not_authenticated_admin();
    ICRC_REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        registry.retain(|canister| canister.ledger != ledger);
    });
}

/// Sets the operator principal.
#[update]
async fn set_operator(operator: Principal) {
    let controllers = get_controllers().await;
    if !controllers.contains(&ic_cdk::caller()) {
        trap("Unauthorized: caller is not a controller");
    }
    CONFIG.with(|config| {
        let mut config = config.borrow_mut();
        config.operator = Some(operator);
    });
}

#[query]
async fn get_all_neurons() -> Vec<NeuronData> {
    NEURON_REGISTRY.with(|registry| {
        let registry = registry.borrow();
        registry.clone().into_iter().collect()
    })
}

#[update]
async fn replace_all_neurons(neurons: Vec<NeuronData>) {
    trap_if_not_authenticated_admin();
    NEURON_REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        registry.clear();
        for neuron in neurons {
            registry.insert(neuron);
        }
    })
}

#[update]
pub async fn allow_signing() {
    get_root_id().await;
    let _ = signer::allow_signing(None).await;
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug, Eq)]
pub struct ICRC1Memory {
    pub index: Option<String>,
    pub ledger: String,
    pub name: String,
    pub logo: Option<String>,
    pub symbol: String,
    pub category: Category,
    pub decimals: u8,
    pub fee: Nat,
    pub root_canister_id: Option<String>,
    pub date_added: Option<u64>,
}

impl Hash for ICRC1Memory {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ledger.hash(state);
    }
}

impl PartialEq for ICRC1Memory {
    fn eq(&self, other: &Self) -> bool {
        self.ledger == other.ledger
    }
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
struct Memory {
    registry: HashSet<ICRC1Memory>,
    neurons: Option<HashSet<NeuronData>>,
    config: Conf,
}

/// Applies changes prior to the canister upgrade.
#[pre_upgrade]
pub fn stable_save() {
    let registry = ICRC_REGISTRY.with(|registry| {
        let registry1 = registry.borrow();
        registry1
            .iter()
            .map(|x| ICRC1Memory {
                index: x.index.clone(),
                ledger: x.ledger.clone(),
                name: x.name.clone(),
                logo: x.logo.clone(),
                symbol: x.symbol.clone(),
                category: x.category.clone(),
                decimals: x.decimals,
                fee: x.fee.clone(),
                root_canister_id: x.root_canister_id.clone(),
                date_added: Some(x.date_added),
            })
            .collect()
    });
    let config = CONFIG.with(|config| {
        let config = config.borrow();
        config.clone()
    });
    let neurons = NEURON_REGISTRY.with(|registry| {
        let registry = registry.borrow();
        registry.clone()
    });
    let mem = Memory {
        registry,
        config,
        neurons: Some(neurons),
    };
    storage::stable_save((mem,))
        .expect("Stable save exited unexpectedly: unable to save data to stable memory.");
}

/// Applies changes following the canister upgrade.
#[post_upgrade]
pub fn stable_restore() {
    let (mo,): (Memory,) = storage::stable_restore()
        .expect("Stable restore exited unexpectedly: unable to restore data from stable memory.");
    CONFIG.with(|mut config| {
        let mut config = config.borrow_mut();
        *config = mo.config.clone();
    });
    timer_service::start_timer(3600);
    ICRC_REGISTRY.with(|mut registry| {
        let mut registry = registry.borrow_mut();
        *registry = mo
            .registry
            .iter()
            .map(|x| ICRC1 {
                index: x.index.clone(),
                ledger: x.ledger.clone(),
                name: x.name.clone(),
                logo: x.logo.clone(),
                symbol: x.symbol.clone(),
                category: x.category.clone(),
                decimals: x.decimals,
                fee: x.fee.clone(),
                root_canister_id: x.root_canister_id.clone(),
                date_added: x.date_added.unwrap_or(ic_cdk::api::time()),
            })
            .collect()
    });
    NEURON_REGISTRY.with(|mut registry| {
        let mut registry = registry.borrow_mut();
        *registry = mo.neurons.unwrap_or_default().clone();
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
            match call(canister, "get_root_by_principal", (princ.to_text(), 0)).await {
                Ok((Some(root_id),)) => root_id,
                Ok((None,)) => trap("No root found for this principal"),
                Err((_, err)) => trap(&format!("Failed to request IM: {}", err)),
            }
        }
    }
}

fn trap_if_not_authenticated_admin() {
    let princ = caller();
    match CONFIG.with(|c| c.borrow_mut().operator) {
        None => trap("Unauthorised"),
        Some(operator) => {
            if !operator.eq(&princ) {
                trap("Unauthorised")
            }
        }
    }
}

async fn get_controllers() -> Vec<Principal> {
    let res: CallResult<(ic_cdk::api::management_canister::main::CanisterStatusResponse,)> = call(
        Principal::management_canister(),
        "canister_status",
        (CanisterIdRequest { canister_id: id() },),
    )
    .await;
    res
        .expect("Get controllers function exited unexpectedly: inter-canister call to management canister for canister_status returned an empty result.")
        .0.settings.controllers
}
