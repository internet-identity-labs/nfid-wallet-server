use asset_util::CertifiedAssets;
use candid::{CandidType, Deserialize};
use canister_sig_util::signature_map::SignatureMap;
use ic_cdk::trap;
use ic_stable_structures::DefaultMemoryImpl;
use internet_identity_interface::internet_identity::types::*;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::time::Duration;

/// Default value for max number of inflight captchas.
pub const DEFAULT_MAX_INFLIGHT_CAPTCHAS: u64 = 500;

/// Default registration rate limit config.
pub const DEFAULT_RATE_LIMIT_CONFIG: RateLimitConfig = RateLimitConfig {
    time_per_token_ns: Duration::from_secs(10).as_nanos() as u64,
    max_tokens: 20_000,
};

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
}

impl Default for State {
    fn default() -> Self {
        Self {
            sigs: RefCell::new(SignatureMap::default()),
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

pub async fn init_salt() {
    //TODO
}
