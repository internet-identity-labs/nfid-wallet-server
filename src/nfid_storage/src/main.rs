use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};

use candid::CandidType;
use candid::{candid_method, Principal};
use ic_cdk::{call, storage, trap};
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};
use serde::{Deserialize, Serialize};

thread_local! {
    static STATE: State = State::default();
    static PASSKEYS: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
    static USER_KEYS: RefCell<HashMap<u64, Vec<String>>> = RefCell::new(HashMap::new());
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
        Self { im_canister: Cell::new(None) }
    }
}

/// Invoked when the canister starts.
/// Initializes the application with `InitArgs` parameters and stores them in persistent storage.
#[init]
#[candid_method(init)]
fn init(maybe_arg: Option<InitArgs>) {
    if maybe_arg.is_some() {
        init_im_canister(
            maybe_arg.expect("The maybe_arg failed after existence check.").im_canister,
        );
    }
}

/// Returns a passkey for the given array of keys.
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

/// Returns the public data of a passkey using the specified key.
#[query]
#[candid_method(query)]
async fn get_passkey_by_anchor(anchor: u64) -> Vec<PasskeyData> {
    let mut response: Vec<PasskeyData> = Vec::new();
    USER_KEYS.with(|user_keys| {
        if let Some(keys_vec) = user_keys.borrow().get(&anchor) {
            PASSKEYS.with(|passkeys| {
                for key in keys_vec {
                    if let Some(value) = passkeys.borrow().get(key) {
                        response.push(PasskeyData { key: key.clone(), data: value.clone() });
                    }
                }
            });
        }
    });
    response
}

/// Persists the public data of a passkey using the specified key.
#[update]
#[candid_method(update)]
async fn store_passkey(key: String, data: String, anchor: u64) -> u64 {
    let caller: Principal = ic_cdk::caller();
    let (option_root, ): (Option<u64>, ) = call(get_im_canister(), "get_anchor_by_principal", (caller.to_text(), ))
        .await
        .expect("Identity Manager canister returned an empty response for the get_anchor_by_principal method.");
    if option_root.is_none()
        || option_root.expect("The option_root failed after existence check.") != anchor
    {
        trap("Unauthorised");
    }
    PASSKEYS.with(|passkeys| {
        let mut passkeys = passkeys.borrow_mut();
        passkeys.insert(key.clone(), data.clone());
    });
    USER_KEYS.with(|user_keys| {
        let mut user_keys = user_keys.borrow_mut();
        if let Some(keys_vec) = user_keys.get_mut(&anchor) {
            if !keys_vec.contains(&key) {
                keys_vec.push(key.clone());
            }
        } else {
            user_keys.insert(anchor, vec![key.clone()]);
        }
    });
    option_root.expect("The option_root failed after existence check.")
}

/// Removes the public data of a passkey using the specified key.
#[update]
#[candid_method(update)]
async fn remove_passkey(key: String, anchor: u64) -> u64 {
    let caller: Principal = ic_cdk::caller();
    let (option_root, ): (Option<u64>, ) = call(get_im_canister(), "get_anchor_by_principal", (caller.to_text(), ))
        .await
        .expect("Identity Manager canister returned an empty response for the get_anchor_by_principal method.");
    if option_root.is_none()
        || option_root.expect("The option_root failed after existence check.") != anchor
    {
        trap("Unauthorised");
    }
    PASSKEYS.with(|passkeys| {
        let mut passkeys = passkeys.borrow_mut();
        passkeys.remove(&key);
    });
    USER_KEYS.with(|user_keys| {
        let mut user_keys = user_keys.borrow_mut();
        if let Some(keys_vec) = user_keys.get_mut(&anchor) {
            keys_vec.retain(|x| x != &key);
        }
    });
    option_root.expect("The option_root failed after existence check.")
}

/// Applies changes after the canister upgrade.
#[post_upgrade]
async fn post_upgrade(maybe_arg: Option<InitArgs>) {
    init_from_memory().await;
}

/// Applies changes before the canister upgrade.
#[pre_upgrade]
async fn save_persistent_state() {
    save_to_temp_memory().await;
}

pub fn init_im_canister(im_canister: Principal) {
    STATE.with(|s| s.im_canister.set(Some(im_canister)));
}

fn main() {}

// Order dependent: do not move above any function annotated with #[candid_method]!
candid::export_service!();

#[derive(Clone, Debug, CandidType, Deserialize)]
struct TempMemory {
    im_canister: Option<Principal>,
    passkeys: Option<HashMap<String, String>>,
    anchors_data: Option<HashMap<u64, Vec<String>>>,
}

pub fn get_im_canister() -> Principal {
    STATE.with(|s| s.im_canister.get().expect("IM canister not set"))
}

pub async fn init_from_memory() {
    let (mo,): (TempMemory,) = storage::stable_restore()
        .expect("Stable restore exited unexpectedly: unable to restore data from stable memory.");
    STATE.with(|s| {
        s.im_canister.set(mo.im_canister);
    });
    PASSKEYS.with(|passkeys| {
        let mut map = passkeys.borrow_mut();
        mo.passkeys.map(|b| map.extend(b));
    });
    USER_KEYS.with(|user_keys| {
        let mut map = user_keys.borrow_mut();
        mo.anchors_data.map(|b| map.extend(b));
    });
}

pub async fn save_to_temp_memory() {
    let (im_canister,) = STATE.with(|s| (s.im_canister.get(),));
    let passkeys = PASSKEYS.with(|passkeys| passkeys.borrow().clone());

    let anchors_data = USER_KEYS.with(|user_keys| user_keys.borrow().clone());

    let mo: TempMemory =
        TempMemory { im_canister, passkeys: Some(passkeys), anchors_data: Some(anchors_data) };
    storage::stable_save((mo,))
        .expect("Stable save exited unexpectedly: unable to save data to stable memory.");
}
