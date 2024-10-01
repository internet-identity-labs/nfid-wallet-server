use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use candid::{candid_method, Principal};
use candid::CandidType;
use ic_cdk::{call, caller, storage, trap};
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};
use serde::{Deserialize, Serialize};

thread_local! {
    static STATE: State = State::default();
    static TRANSACTIONS: RefCell<HashMap<String, Vec<SwapTransaction>>> = RefCell::new(HashMap::new());
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct InitArgs {
    pub im_canister: Principal,
}



#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct SwapTransaction {
    pub start_time: u64,
    pub end_time: u64,
    pub transfer_id: u64,
    pub transfer_nfid_id: u64,
    pub deposit: u64,
    pub swap: u64,
    pub withdraw: u64,
    pub error: Option<String>,
    pub stage: SwapStage,
    pub source_ledger: String,
    pub target_ledger: String,
    pub source_amount: u64,
    pub target_amount: u64
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub enum SwapStage {
    Transfer,
    Deposit,
    Swap,
    Withdraw,
    Completed,
    Error,
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

#[init]
#[candid_method(init)]
fn init(maybe_arg: Option<InitArgs>) {
    if maybe_arg.is_some() {
        init_im_canister(maybe_arg.unwrap().im_canister);
    }
}

#[query]
#[candid_method(query)]
async fn get_transactions(caller: String) -> Vec<SwapTransaction> {
    TRANSACTIONS.with(|trss| {
            if let Some(transactions) = trss.borrow().get(&caller) {
                transactions.clone()
            } else {
                Vec::new()
            }
    })
}


#[update]
#[candid_method(update)]
async fn store_transaction(data: SwapTransaction) {
    let caller = caller().to_text();
    TRANSACTIONS.with(|trss| {
        let mut transactions = trss.borrow_mut();
        transactions.entry(caller).or_insert_with(Vec::new).push(data.clone());
    })
}

#[post_upgrade]
async fn post_upgrade(maybe_arg: Option<InitArgs>) {
    init_from_memory().await;
}

#[pre_upgrade]
async fn save_persistent_state() {
    save_to_temp_memory().await;
}


pub fn init_im_canister(im_canister: Principal) {
    STATE.with(|s| {
        s.im_canister.set(Some(im_canister))
    });
}

fn main() {}

// Order dependent: do not move above any function annotated with #[candid_method]!
candid::export_service!();

#[derive(Clone, Debug, CandidType, Deserialize)]
struct TempMemory {
    im_canister: Option<Principal>,
    transactions: Option<HashMap<String, Vec<SwapTransaction>>>,
}


pub fn get_im_canister() -> Principal {
    STATE.with(|s| {
        s.im_canister.get().expect("IM canister not set")
    })
}

pub async fn init_from_memory() {
    let (mo, ): (TempMemory, ) = storage::stable_restore().unwrap();
    STATE.with(|s| {
        s.im_canister.set(mo.im_canister);
    });
    TRANSACTIONS.with(|trss| {
        let mut map = trss.borrow_mut();
        mo.transactions.map(|b| map.extend(b));
    });
}

pub async fn save_to_temp_memory() {
    let (im_canister, ) = STATE.with(|s| {
        (s.im_canister.get(), )
    });
    let trss = TRANSACTIONS.with(|trs| {
        trs.borrow().clone()
    });

    let mo: TempMemory = TempMemory { im_canister, transactions: Some(trss) };
    storage::stable_save((mo, )).unwrap();
}


#[ic_cdk_macros::query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    __export_service()
}