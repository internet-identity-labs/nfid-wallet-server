use candid::{candid_method, Principal};
use candid::CandidType;
use canister_sig_util::signature_map::LABEL_SIG;
use ic_cdk::{call, print, trap};
use ic_cdk::api::set_certified_data;
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};
use internet_identity_interface::internet_identity::types::*;
use serde::{Deserialize, Serialize};

use crate::state::{get_im_canister, init_from_memory, init_im_canister, Salt, save_to_temp_memory};

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

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct InitArgs {
    pub im_canister: Principal,
}

#[update]
#[candid_method]
async fn init_salt() {
    state::init_salt().await;
}

#[query(composite = true)]
#[candid_method(query)]
async fn get_principal(anchor_number: AnchorNumber, frontend: FrontendHostname) -> Principal {
    let caller: Principal = ic_cdk::caller();
    let (option_root, ): (Option<u64>, ) = call(get_im_canister(), "get_anchor_by_principal", (caller.to_text(), ))
        .await
        .expect("Identity Manager canister returned an empty response for the get_anchor_by_principal method.");
    if option_root.is_none() || option_root.expect("The option_root is empty for the get_anchor_by_principal method call.") != anchor_number {
        trap("Unauthorised");
    }
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
        targets,
    )
        .await
}

#[query(composite = true)]
#[candid_method(query)]
async fn get_delegation(
    anchor_number: AnchorNumber,
    frontend: FrontendHostname,
    session_key: SessionKey,
    expiration: Timestamp,
    targets: Option<Vec<Principal>>,
) -> GetDelegationResponse {
    let delegation = delegation::get_delegation(anchor_number, frontend, session_key, expiration, targets);
    let caller: Principal = ic_cdk::caller();
    let (option_root, ): (Option<u64>, ) = call(get_im_canister(), "get_anchor_by_principal", (caller.to_text(), ))
        .await.expect("Identity Manager canister returned an empty response for the get_anchor_by_principal method.");
    if option_root.is_none() || option_root.expect("The option_root is empty for the get_anchor_by_principal method call.") != anchor_number {
        trap("Unauthorised");
    }
    delegation
}

#[query]
async fn get_im_canister_setting() -> Principal {
    state::get_im_canister()
}

#[init]
#[candid_method(init)]
fn init(maybe_arg: Option<InitArgs>) {
    initialize(maybe_arg);
}

#[post_upgrade]
async fn post_upgrade(maybe_arg: Option<InitArgs>) {
    init_from_memory().await;
    initialize(maybe_arg);
}

#[pre_upgrade]
async fn save_persistent_state() {
    save_to_temp_memory().await;
}

fn initialize(maybe_arg: Option<InitArgs>) {
    update_root_hash();
    if let Some(args) = maybe_arg {
        init_im_canister(args.im_canister);
    }
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
