use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::time::Duration;

use asset_util::CertifiedAssets;
use candid::{CandidType};
use serde::{Deserialize, Serialize};
use canister_sig_util::signature_map::SignatureMap;
use ic_cdk::{storage, trap};
use ic_stable_structures::DefaultMemoryImpl;
use internet_identity_interface::internet_identity::types::*;

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
}

//TODO move to stable
#[derive(Clone, Debug, CandidType, Deserialize)]
struct TempMemory {
    salt: Option<Salt>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            sigs: RefCell::new(SignatureMap::default()),
            salt: Cell::new(None),
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

pub async fn ensure_salt_set() {
    if STATE.with(|s| s.salt.get()).is_none() {
        trap("Salt not set")
    }
}
pub fn get_salt() -> Salt {
    STATE.with(|s| {
        s.salt.get().expect("Salt not set")
    })
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

pub async fn init_from_memory() {
    let (mo, ): (TempMemory, ) = storage::stable_restore().unwrap();
    STATE.with(|s| {
        s.salt.set(mo.salt);
    });
}

pub async fn save_to_temp_memory() {
    let salt = STATE.with(|s| {
        s.salt.get()
    });
    let mo: TempMemory = TempMemory { salt };
    storage::stable_save((mo, )).unwrap();
}