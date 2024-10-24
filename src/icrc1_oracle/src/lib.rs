use core::hash::Hash;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use candid::{CandidType, Nat, Principal};
use candid::{candid_method, export_service};
use ic_cdk::{call, caller, id, storage, trap};
use ic_cdk::api::call::CallResult;
use ic_cdk_macros::*;
use serde::{Deserialize, Serialize};

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
    pub controllers: Option<Vec<Principal>>,
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
        controllers: Default::default(),
        im_canister: None
    });
    pub static ICRC_REGISTRY: RefCell<HashSet<ICRC1>> = RefCell::new(HashSet::default());
}

/// Synchronizes the controllers from the management canister.
/// This ensures the canister is aware of all controllers, allowing them to function as administrators.
#[update]
#[candid_method(update)]
async fn sync_controllers() -> Vec<String> {
    let res: CallResult<(CanisterStatusResponse, )> = call(
        Principal::management_canister(),
        "canister_status",
        (CanisterIdRequest {
            canister_id: id(),
        }, ),
    ).await;

    let controllers = res
        .expect("Sync controller function exited unexpectedly: inter-canister call to management canister for canister_status returned an empty result.")
        .0.settings.controllers;
    CONFIG.with(|c| c.borrow_mut().controllers.replace(controllers.clone()));
    controllers.iter().map(|x| x.to_text()).collect()
}

/// Persists a single ICRC1 canister's metadata into the canister's storage.
#[update]
pub async fn store_icrc1_canister(request: ICRC1Request) {
    get_root_id().await;
    Principal::from_text(request.ledger.clone()).unwrap_or_else(|_| {
        trap("Invalid ledger principal");
    });
    if request.index.is_some() {
       Principal::from_text(request.index.clone().expect("The request.index failed after existence check.")).unwrap_or_else(|_| {
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
        for canister in icrc1 {
            registry.insert(canister);
        }
    })
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
struct Memory {
    registry: HashSet<ICRC1>,
    config: Conf,
}

/// Applies changes prior to the canister upgrade.
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
    let mem = Memory {
        registry,
        config,
    };
    storage::stable_save((mem, ))
        .expect("Stable save exited unexpectedly: unable to save data to stable memory.");
}

/// Applies changes following the canister upgrade.
#[post_upgrade]
pub fn stable_restore() {
    let (mo, ): (Memory, ) = storage::stable_restore()
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
            match call(canister, "get_root_by_principal", (princ.to_text(), 0)).await {
                Ok((Some(root_id), )) => root_id,
                Ok((None, )) => trap("No root found for this principal"),
                Err((_, err)) => trap(&format!("Failed to request IM: {}", err)),
            }
        }
    }
}


fn trap_if_not_authenticated_admin() {
    let princ = caller();
    match CONFIG.with(|c| c.borrow_mut().controllers.clone())
    {
        None => {
            trap("Unauthorised")
        }
        Some(controllers) => {
            if !controllers.contains(&princ) {
                trap("Unauthorised")
            }
        }
    }
}

