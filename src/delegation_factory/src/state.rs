use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::time::Duration;

use asset_util::CertifiedAssets;
use candid::{CandidType, Principal};
use canister_sig_util::signature_map::SignatureMap;
use ic_cdk::{storage, trap};
use ic_stable_structures::DefaultMemoryImpl;
use internet_identity_interface::internet_identity::types::*;
use serde::{Deserialize, Serialize};

use crate::random_salt;

pub type Salt = [u8; 32];

thread_local! {
    static STATE: State = State::default();
    static ASSETS: RefCell<CertifiedAssets> = RefCell::new(CertifiedAssets::default());
}

#[cfg(not(test))]
fn time() -> Timestamp {
    ic_cdk::api::time()
}

struct State {
    sigs: RefCell<SignatureMap>,
    salt: Cell<Option<Salt>>,
    im_canister: Cell<Option<Principal>>,
    operator: Cell<Option<Principal>>,
}

//TODO move to stable
#[derive(Clone, Debug, CandidType, Deserialize)]
struct TempMemory {
    salt: Option<Salt>,
    im_canister: Option<Principal>,
    operator: Option<Principal>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            sigs: RefCell::new(SignatureMap::default()),
            salt: Cell::new(None),
            im_canister: Cell::new(None),
            operator: Cell::new(None),
        }
    }
}

pub fn assets_mut<R>(f: impl FnOnce(&mut CertifiedAssets) -> R) -> R {
    ASSETS.with(|assets| f(&mut assets.borrow_mut()))
}

pub fn assets_and_signatures<R>(f: impl FnOnce(&CertifiedAssets, &SignatureMap) -> R) -> R {
    ASSETS.with(|assets| STATE.with(|s| f(&assets.borrow(), &s.sigs.borrow())))
}

pub fn signature_map<R>(f: impl FnOnce(&SignatureMap) -> R) -> R {
    STATE.with(|s| f(&s.sigs.borrow()))
}

pub fn signature_map_mut<R>(f: impl FnOnce(&mut SignatureMap) -> R) -> R {
    STATE.with(|s| f(&mut s.sigs.borrow_mut()))
}

pub fn ensure_settings_set() {
    if STATE.with(|s| s.salt.get()).is_none() {
        trap("Salt not set")
    }
    if STATE.with(|s| s.im_canister.get()).is_none() {
        trap("IM canister not set")
    }
}
pub fn get_salt() -> Salt {
    STATE.with(|s| s.salt.get().expect("Salt not set"))
}

pub fn clean_state() {
    STATE.with(|s| {
        s.sigs.replace(SignatureMap::default());
    });
    ASSETS.with(|assets| {
        assets.replace(CertifiedAssets::default());
    });
}

pub fn get_im_canister() -> Principal {
    STATE.with(|s| s.im_canister.get().expect("IM canister not set"))
}

pub async fn init_salt() {
    let salt = random_salt().await;
    STATE.with(|s| {
        if s.salt.get().is_some() {
            trap("Salt already set");
        }
        s.salt.set(Some(salt))
    });
}

pub fn init_im_canister(im_canister: Principal) {
    STATE.with(|s| s.im_canister.set(Some(im_canister)));
}

pub fn set_operator(operator: Principal) {
    STATE.with(|s| s.operator.set(Some(operator)));
}

pub fn get_operator() -> Principal {
    STATE.with(|s| s.operator.get().expect("Operator not set"))
}

pub async fn init_from_memory() {
    let (mo,): (TempMemory,) = storage::stable_restore()
        .expect("Stable restore exited unexpectedly: unable to restore data from stable memory.");
    STATE.with(|s| {
        s.salt.set(mo.salt);
        s.im_canister.set(mo.im_canister);
        s.operator.set(mo.operator);
    });
}

pub async fn save_to_temp_memory() {
    let (salt, im_canister, operator) =
        STATE.with(|s| (s.salt.get(), s.im_canister.get(), s.operator.get()));
    let mo: TempMemory = TempMemory { salt, im_canister, operator };
    storage::stable_save((mo,))
        .expect("Stable save exited unexpectedly: unable to save data to stable memory.");
}
