use std::cell::{Cell, RefCell};

use candid::{candid_method, Principal};
use candid::CandidType;
use hex::encode;
use ic_cdk::{call, caller, storage, trap};
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, update};
use serde::Deserialize;
use sha2::{Digest, Sha256};
thread_local! {
    static STATE: State = State::default();
}


#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct InitArgs {
    pub im_canister: Principal,
    pub salt: String,
    pub ecdsa_salt: String,
}


struct State {
    im_canister: Cell<Option<Principal>>,
    salt: RefCell<String>,
    ecdsa_salt: RefCell<String>,
}


impl Default for State {
    fn default() -> Self {
        Self {
            im_canister: Cell::new(None),
            salt: RefCell::new("".to_string()),
            ecdsa_salt: RefCell::new("".to_string()),
        }
    }
}

/// Invoked when the canister starts.
/// Initializes the application with `InitArgs` parameters and stores them in persistent storage.
#[init]
#[candid_method(init)]
fn init(maybe_arg: Option<InitArgs>) {
    if let Some(arg) = maybe_arg {
        init_im_canister(arg);
    }
}


#[update]
async fn get_salt() -> String {
    let root = get_root_id().await;
    let ecdsa_salt = STATE.with(|s| s.ecdsa_salt.borrow().clone());
    let salted_key = format!("{}{}", root, ecdsa_salt);
    sha2(&salted_key)
}

#[update]
async fn get_anon_salt(data: String) -> String {
    let root = get_root_id().await;
    let ecdsa_salt = STATE.with(|s| s.ecdsa_salt.borrow().clone());
    let salt = STATE.with(|s: &State| s.salt.borrow().clone());
    let salted_data = format!("{}{}{}{}", root, data, salt, ecdsa_salt);
    sha2(&salted_data)
}


/// Applies changes afterr the canister upgrade.
#[post_upgrade]
async fn post_upgrade(_maybe_arg: Option<InitArgs>) {
    init_from_memory().await;
}

/// Applies changes before the canister upgrade.
#[pre_upgrade]
async fn save_persistent_state() {
    save_to_temp_memory().await;
}


pub fn init_im_canister(args: InitArgs) {
    STATE.with(|s: &State| {
        s.im_canister.set(Some(args.im_canister));
        s.ecdsa_salt.replace(args.ecdsa_salt);
        s.salt.replace(args.salt);
    });
}

fn main() {}

// Order dependent: do not move above any function annotated with #[candid_method]!
candid::export_service!();

#[derive(Clone, Debug, CandidType, Deserialize)]
struct TempMemory {
    im_canister: Option<Principal>,
    ecdsa_salt: Option<String>,
    salt: Option<String>,
}


pub fn get_im_canister() -> Principal {
    STATE.with(|s: &State| {
        s.im_canister.get().expect("IM canister not set")
    })
}

pub async fn init_from_memory() {
    let (mo, ): (TempMemory,) = storage::stable_restore()
        .expect("Stable restore failed: unable to restore data from stable memory.");
    STATE.with(|s: &State| {
        s.im_canister.set(mo.im_canister);
        s.ecdsa_salt.replace(mo.ecdsa_salt.unwrap());
        s.salt.replace(mo.salt.unwrap());
    });
}

pub async fn save_to_temp_memory() {
    let temp_memory = TempMemory {
        im_canister: STATE.with(|s| s.im_canister.get()),
        ecdsa_salt: STATE.with(|s| Some(s.ecdsa_salt.take())),
        salt: STATE.with(|s| Some(s.salt.take())),
    };
    storage::stable_save((temp_memory,))
        .expect("Stable save failed: unable to save data to stable memory.");
}

fn sha2(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value);
    encode(hasher.finalize()) // Convert bytes to hex string
}

async fn get_root_id() -> String {
    match STATE.with(|c| c.im_canister.get()) {
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
