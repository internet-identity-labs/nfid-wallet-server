use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use candid::{candid_method, Principal};
use candid::CandidType;
use ic_cdk::{call, storage, trap};
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};
use serde::{Deserialize, Serialize};

thread_local! {
    static STATE: State = State::default();
    static PASSKEYS: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}


#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct InitArgs {
    pub im_canister: Principal,
}


#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct PasskeyData {
    pub key: String,
    pub data: String,
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
        init_im_canister(maybe_arg.expect("The maybe_arg failed after existence check.").im_canister);
    }
}

#[query]
#[candid_method(query)]
async fn get_passkey(keys: Vec<String>) -> Vec<PasskeyData> {
    let mut response: Vec<PasskeyData> = Vec::new();
    PASSKEYS.with(|passkeys| {
        for key in keys {
            if let Some(value) = passkeys.borrow().get(&key) {
                response.push(PasskeyData { key, data: value.clone() });
            }
        }
    });
    response
}


#[update]
#[candid_method(update)]
async fn store_passkey(key: String, data: String) -> u64 {
    let caller: Principal = ic_cdk::caller();
    let (option_root, ): (Option<u64>, ) = call(get_im_canister(), "get_anchor_by_principal", (caller.to_text(), ))
        .await
        .expect("Identity Manager canister returned an empty response for the get_anchor_by_principal method.");
    if option_root.is_none() {
        trap("Unauthorised");
    }
    PASSKEYS.with(|passkeys| {
        let mut passkeys = passkeys.borrow_mut();
        passkeys.insert(key.clone(), data.clone());
        option_root.expect("The option_root failed after existence check.")
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
    passkeys: Option<HashMap<String, String>>,
}


pub fn get_im_canister() -> Principal {
    STATE.with(|s| {
        s.im_canister.get().expect("IM canister not set")
    })
}

pub async fn init_from_memory() {
    let (mo, ): (TempMemory, ) = storage::stable_restore()
        .expect("Stable restore exited unexpectedly: unable to restore data from stable memory.");
    STATE.with(|s| {
        s.im_canister.set(mo.im_canister);
    });
    PASSKEYS.with(|passkeys| {
        let mut map = passkeys.borrow_mut();
        mo.passkeys.map(|b| map.extend(b));
    });
}

pub async fn save_to_temp_memory() {
    let (im_canister, ) = STATE.with(|s| {
        (s.im_canister.get(), )
    });
    let passkeys = PASSKEYS.with(|passkeys| {
        passkeys.borrow().clone()
    });

    let mo: TempMemory = TempMemory { im_canister, passkeys: Some(passkeys) };
    storage::stable_save((mo, ))
        .expect("Stable save exited unexpectedly: unable to save data to stable memory.");
}