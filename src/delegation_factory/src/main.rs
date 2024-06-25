use candid::{candid_method, Principal};
use ic_cdk::api::set_certified_data;
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};

use canister_sig_util::signature_map::LABEL_SIG;
use ic_cdk::{call, trap};
use internet_identity_interface::internet_identity::types::*;
use crate::state::{init_from_memory, Salt, save_to_temp_memory};

/// Type conversions between internal and external types.
mod delegation;
mod hash;
mod state;
// mod storage;

// Some time helpers
const fn secs_to_nanos(secs: u64) -> u64 {
    secs * 1_000_000_000
}

const MINUTE_NS: u64 = secs_to_nanos(60);
const HOUR_NS: u64 = 60 * MINUTE_NS;
const DAY_NS: u64 = 24 * HOUR_NS;


#[update]
#[candid_method]
async fn init_salt() {
    state::init_salt().await;
}

#[query]
#[candid_method(query)]
fn get_principal(anchor_number: AnchorNumber, frontend: FrontendHostname) -> Principal {
    delegation::get_principal(anchor_number, frontend)
}

#[update]
#[candid_method]
async fn prepare_delegation(
    anchor_number: AnchorNumber,
    frontend: FrontendHostname,
    session_key: SessionKey,
    max_time_to_live: Option<u64>,
    targets: Option<Vec<Principal>>,
) -> (UserKey, Timestamp) {
    delegation::prepare_delegation(
        anchor_number,
        frontend,
        session_key,
        max_time_to_live,
        targets
    )
        .await
}

#[query]
#[candid_method(query)]
fn get_delegation(
    anchor_number: AnchorNumber,
    frontend: FrontendHostname,
    session_key: SessionKey,
    expiration: Timestamp,
    targets: Option<Vec<Principal>>,
) -> GetDelegationResponse {
    delegation::get_delegation(anchor_number, frontend, session_key, expiration, targets)
}

#[init]
#[candid_method(init)]
fn init(maybe_arg: Option<InternetIdentityInit>) {
    initialize(maybe_arg);
}

#[post_upgrade]
async fn post_upgrade(maybe_arg: Option<InternetIdentityInit>) {
    init_from_memory().await;
    initialize(maybe_arg);
}

#[pre_upgrade]
async fn save_persistent_state() {
    save_to_temp_memory().await;
}

fn initialize(maybe_arg: Option<InternetIdentityInit>) {
    update_root_hash();
}


fn update_root_hash() {
    use ic_certification::{fork_hash, labeled_hash};
    state::assets_and_signatures(|assets, sigs| {
        let prefixed_root_hash = fork_hash(
            &assets.root_hash(),
            // NB: sigs have to be added last due to lexicographic order of labels
            &labeled_hash(LABEL_SIG, &sigs.root_hash()),
        );
        set_certified_data(&prefixed_root_hash[..]);
    })
}

async fn random_salt() -> Salt {
    let res: Vec<u8> = match call(Principal::management_canister(), "raw_rand", ()).await {
        Ok((res, )) => res,
        Err((_, err)) => trap(&format!("failed to get salt: {err}")),
    };
    let salt: Salt = res[..].try_into().unwrap_or_else(|_| {
        trap(&format!(
            "expected raw randomness to be of length 32, got {}",
            res.len()
        ));
    });
    salt
}

fn main() {}

// Order dependent: do not move above any function annotated with #[candid_method]!
candid::export_service!();
