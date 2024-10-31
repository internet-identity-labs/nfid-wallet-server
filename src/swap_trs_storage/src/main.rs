use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use candid::{CandidType, Nat};
use candid::{candid_method, Principal};
use ic_cdk::{call, caller, storage, trap};
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};
use serde::{Deserialize, Serialize};

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
    static TRANSACTIONS: RefCell<HashMap<String, HashSet<SwapTransaction>>> = RefCell::new(HashMap::new());
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct InitArgs {
    pub im_canister: Principal,
}


#[derive(Clone, Debug, CandidType, Deserialize, Serialize, Eq)]
pub struct SwapTransaction {
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub transfer_id: Option<u64>,
    pub transfer_nfid_id: Option<u64>,
    pub deposit: Option<Nat>,
    pub swap: Option<Nat>,
    pub withdraw: Option<Nat>,
    pub error: Option<String>,
    pub stage: SwapStage,
    pub source_ledger: String,
    pub target_ledger: String,
    pub source_amount: Nat,
    pub target_amount: Nat,
    pub uid: String,
}

impl Hash for SwapTransaction {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uid.hash(state);
    }
}

impl PartialEq for SwapTransaction {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq, Eq)]
pub enum SwapStage {
    TransferNFID,
    TransferSwap,
    Deposit,
    Swap,
    Withdraw,
    Completed,
}


struct State {
    im_canister: Cell<Option<Principal>>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            im_canister: Cell::new(None),
        }
    }
}

/// Invoked when the canister starts.
/// Initializes the application with `InitArgs` parameters and stores them in persistent storage.
#[init]
#[candid_method(init)]
fn init(maybe_arg: Option<InitArgs>) {
    if maybe_arg.is_some() {
        init_im_canister(maybe_arg.expect("The maybe_arg failed after existence check.").im_canister);
    }
}


/// Returns all transactions for the specified caller ID.
#[query]
#[candid_method(query)]
async fn get_transactions(caller: String) -> HashSet<SwapTransaction> {
    TRANSACTIONS.with(|trss| {
        if let Some(transactions) = trss.borrow().get(&caller) {
            transactions.clone()
        } else {
            HashSet::new()
        }
    })
}

/// Persists a transaction for the caller using the specified data.
#[update]
#[candid_method(update)]
async fn store_transaction(data: SwapTransaction) {
    let id = get_root_id().await;
    TRANSACTIONS.with(|trss| {
        let mut transactions = trss.borrow_mut();
        transactions.entry(id).or_insert(HashSet::new()).replace(data.clone());
    })
}

/// Applies changes after the canister upgrade.
#[post_upgrade]
async fn post_upgrade(_: Option<InitArgs>) {
    init_from_memory().await;
}

/// Applies changes before the canister upgrade.
#[pre_upgrade]
async fn save_persistent_state() {
    save_to_temp_memory().await;
}


pub fn init_im_canister(im_canister: Principal) {
    STATE.with(|s| {
        s.borrow().im_canister.set(Some(im_canister))
    });
}

fn main() {}

// Order dependent: do not move above any function annotated with #[candid_method]!
candid::export_service!();

#[derive(Clone, Debug, CandidType, Deserialize)]
struct TempMemory {
    im_canister: Option<Principal>,
    transactions: Option<HashMap<String, HashSet<SwapTransaction>>>,
}


pub fn get_im_canister() -> Principal {
    STATE.with(|s| {
        s.borrow().im_canister.get().expect("IM canister not set")
    })
}

pub async fn init_from_memory() {
    let (mo, ): (TempMemory,) = storage::stable_restore()
        .expect("Stable restore exited unexpectedly: unable to restore data from stable memory.");
    STATE.with(|s| {
        s.borrow_mut().im_canister.set(mo.im_canister);
    });
    TRANSACTIONS.with(|trss| {
        let mut map = trss.borrow_mut();
        mo.transactions.map(|b| map.extend(b));
    });
}

pub async fn save_to_temp_memory() {
    let (im_canister, ) = STATE.with(|s| {
        (s.borrow_mut().im_canister.get(),)
    });
    let trss = TRANSACTIONS.with(|trs| {
        trs.borrow().clone()
    });

    let mo: TempMemory = TempMemory { im_canister, transactions: Some(trss) };
    storage::stable_save((mo,))
        .expect("Stable save exited unexpectedly: unable to save data to stable memory.");
}


#[ic_cdk_macros::query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    __export_service()
}


async fn get_root_id() -> String {
    match STATE.with(|c| c.borrow_mut().im_canister.get()) {
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