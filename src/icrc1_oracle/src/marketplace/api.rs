//! Canister endpoints of the ICRC-7 marketplace.

use std::time::Duration;

use candid::Principal;
use ic_cdk::caller;
use ic_cdk_macros::{query, update};

use super::env::{Env, IcEnv};
use super::service::{self, ESCROW_SUBACCOUNT};
use super::storage;
use super::types::*;
use crate::{get_root_id, is_operator, trap_if_not_authenticated_admin};

/// How often the clock retries pending purchases.
const CLOCK_INTERVAL: Duration = Duration::from_secs(5 * 60);

pub fn start_clock() {
    ic_cdk_timers::set_timer_interval(CLOCK_INTERVAL, || {
        ic_cdk::spawn(async {
            service::run_clock(&IcEnv).await;
        })
    });
}

// ── Collections ────────────────────────────────────────────────────────────

/// Adds an ICRC-7 collection (as Community). No-op if it is already known.
#[update]
pub async fn store_nft_collection(request: NftCollectionRequest) {
    get_root_id().await;
    service::store_collection(request, IcEnv.now());
}

#[query]
pub fn get_nft_collection(canister_id: Principal) -> Option<NftCollection> {
    storage::get_collection(&canister_id)
}

#[query]
pub fn count_nft_collections() -> u64 {
    storage::COLLECTIONS.with(|c| c.borrow().len() as u64)
}

#[query]
pub fn get_nft_collections_paginated(offset: u64, limit: u64) -> Vec<NftCollection> {
    storage::COLLECTIONS.with(|c| {
        c.borrow()
            .values()
            .skip(offset as usize)
            .take(limit as usize)
            .cloned()
            .collect()
    })
}

/// Upserts collections as given (admin).
#[update]
pub fn replace_nft_collections(collections: Vec<NftCollection>) {
    trap_if_not_authenticated_admin();
    storage::COLLECTIONS.with(|c| {
        let mut c = c.borrow_mut();
        for col in collections {
            c.insert(col.canister_id, col);
        }
    });
}

#[update]
pub fn set_nft_collection_status(canister_id: Principal, status: NftCollectionStatus) {
    trap_if_not_authenticated_admin();
    storage::COLLECTIONS.with(|c| match c.borrow_mut().get_mut(&canister_id) {
        Some(col) => col.status = status,
        None => ic_cdk::trap("Collection not found"),
    });
}

#[update]
pub fn remove_nft_collection(canister_id: Principal) {
    trap_if_not_authenticated_admin();
    storage::COLLECTIONS.with(|c| c.borrow_mut().remove(&canister_id));
}

// ── Config ─────────────────────────────────────────────────────────────────

#[update]
pub fn set_marketplace_config(config: MarketplaceConfig) {
    trap_if_not_authenticated_admin();
    if config.max_retries == 0 {
        ic_cdk::trap("max_retries must be > 0");
    }
    storage::CONFIG.with(|c| *c.borrow_mut() = Some(config));
}

#[query]
pub fn get_marketplace_config() -> Option<MarketplaceConfig> {
    storage::config()
}

/// Account holding buyers' ICP while purchases are in progress.
#[query]
pub fn get_marketplace_escrow() -> Account {
    Account {
        owner: ic_cdk::id(),
        subaccount: Some(ESCROW_SUBACCOUNT.to_vec()),
    }
}

// ── Listings ───────────────────────────────────────────────────────────────

/// Lists an NFT. The caller must own it and have approved this canister via
/// `icrc37_approve_tokens` beforehand.
#[update]
pub async fn create_listing(arg: CreateListingArg) -> MarketResult<Listing> {
    service::create_listing(&IcEnv, caller(), arg).await
}

/// Cancels a listing (seller or admin).
#[update]
pub fn cancel_listing(listing_id: u64) -> MarketResult<Listing> {
    let c = caller();
    service::cancel_listing(c, listing_id, is_operator(&c), IcEnv.now())
}

#[query]
pub fn get_listing(listing_id: u64) -> Option<Listing> {
    storage::get_listing(listing_id)
}

/// Listings on sale in tradable collections, optionally one collection,
/// sorted (default Newest) and paged (`limit` capped at 100).
#[query]
pub fn get_listings(query: ListingQuery) -> ListingPage {
    service::query_listings(query)
}

/// All listings of a seller in any status, newest first.
#[query]
pub fn get_listings_by_seller(seller: Principal, offset: u64, limit: u64) -> Vec<Listing> {
    storage::listings_desc(|l| l.seller == seller, offset, limit)
}

// ── Purchases ──────────────────────────────────────────────────────────────

/// Buys a listing. The caller must first `icrc2_approve` this canister on the
/// ICP ledger for `price_e8s + fee`.
#[update]
pub async fn buy_listing(listing_id: u64) -> MarketResult<MarketTx> {
    service::buy(&IcEnv, caller(), listing_id).await
}

#[query]
pub fn get_marketplace_tx(tx_id: u64) -> Option<MarketTx> {
    storage::get_tx(tx_id)
}

/// Purchases where `user` is the buyer or the seller, newest first.
#[query]
pub fn get_marketplace_txs_by_user(user: Principal, offset: u64, limit: u64) -> Vec<MarketTx> {
    storage::txs_desc(|t| t.buyer == user || t.seller == user, offset, limit)
}

#[query]
pub fn count_marketplace_txs() -> u64 {
    storage::count_txs()
}

// ── Admin ──────────────────────────────────────────────────────────────────

/// All purchases, newest first (admin).
#[query]
pub fn get_marketplace_txs_paginated(offset: u64, limit: u64) -> Vec<MarketTx> {
    trap_if_not_authenticated_admin();
    storage::txs_desc(|_| true, offset, limit)
}

/// Purchases that are not finished, including those flagged for review (admin).
#[query]
pub fn get_pending_marketplace_txs() -> Vec<MarketTx> {
    trap_if_not_authenticated_admin();
    storage::pending_tx_ids()
        .into_iter()
        .filter_map(storage::get_tx)
        .collect()
}

#[update]
pub async fn retry_marketplace_tx(tx_id: u64, reset_dedup: bool) -> MarketResult<MarketTx> {
    trap_if_not_authenticated_admin();
    service::admin_retry_tx(&IcEnv, tx_id, reset_dedup).await
}

#[update]
pub fn close_marketplace_tx(tx_id: u64, note: String) -> MarketResult<MarketTx> {
    trap_if_not_authenticated_admin();
    service::admin_close_tx(tx_id, note, IcEnv.now())
}

/// Runs the retry clock now instead of waiting for the timer (admin).
#[update]
pub async fn run_marketplace_clock() -> u32 {
    trap_if_not_authenticated_admin();
    service::run_clock(&IcEnv).await
}
