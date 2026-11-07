//! Listing and purchase logic.
//!
//! Purchase steps (`TxStatus` names the step that runs next):
//!   PaymentPending     buyer --ICP--> escrow        (icrc2_transfer_from)
//!   NftTransferPending seller --NFT--> buyer        (icrc37_transfer_from)
//!   PayoutPending      escrow --ICP--> seller       (icrc1_transfer)
//!   RefundPending      escrow --ICP--> buyer        (icrc1_transfer)
//!
//! Each step stores its `created_at_time` before the first attempt and reuses
//! it on retries, so a repeat that already executed comes back as a duplicate.
//! A step that cannot finish now stays pending; the clock retries it.

use candid::{Nat, Principal};

use super::env::{Env, IcpTransfer, IcpTransferFrom, NftTransferFrom, TransferOutcome};
use super::storage::{self, ClockGuard, TxGuard};
use super::types::*;

/// Escrow subaccount of the oracle that holds buyers' ICP during a purchase.
pub const ESCROW_SUBACCOUNT: [u8; 32] = *b"nfid-marketplace-escrow\0\0\0\0\0\0\0\0\0";

/// Max txs one clock run processes.
const CLOCK_BATCH: usize = 50;

#[derive(Clone, Copy)]
enum MemoStep {
    Payment = 1,
    Nft = 2,
    Payout = 3,
    Refund = 4,
}

/// 16 bytes: "NFIDMKT" + step + tx id (big endian).
fn memo(step: MemoStep, tx_id: u64) -> Vec<u8> {
    let mut m = b"NFIDMKT".to_vec();
    m.push(step as u8);
    m.extend_from_slice(&tx_id.to_be_bytes());
    m
}

fn escrow(env: &impl Env) -> Account {
    Account {
        owner: env.self_id(),
        subaccount: Some(ESCROW_SUBACCOUNT.to_vec()),
    }
}

fn require_caller(caller: Principal) -> MarketResult<()> {
    if caller == Principal::anonymous() {
        Err(MarketplaceError::AnonymousCaller)
    } else {
        Ok(())
    }
}

fn require_config() -> MarketResult<MarketplaceConfig> {
    storage::config().ok_or(MarketplaceError::NotConfigured)
}

fn require_tradable(collection: &Principal) -> MarketResult<NftCollection> {
    let c = storage::get_collection(collection).ok_or(MarketplaceError::CollectionNotFound)?;
    if !c.status.is_tradable() {
        return Err(MarketplaceError::CollectionNotAllowed);
    }
    Ok(c)
}

fn set_listing_status(listing_id: u64, status: ListingStatus, now: u64) {
    if let Some(mut l) = storage::get_listing(listing_id) {
        l.status = status;
        l.updated_at = now;
        storage::save_listing(&l);
    }
}

// ── Collections ────────────────────────────────────────────────────────────

/// Adds a collection as Community. An existing entry is left untouched so
/// users cannot overwrite metadata or status set by an admin.
pub fn store_collection(req: NftCollectionRequest, now: u64) {
    storage::COLLECTIONS.with(|c| {
        c.borrow_mut()
            .entry(req.canister_id)
            .or_insert_with(|| NftCollection {
                canister_id: req.canister_id,
                name: req.name,
                symbol: req.symbol,
                logo: req.logo,
                description: req.description,
                status: NftCollectionStatus::Community,
                date_added: now,
            });
    });
}

// ── Listings ───────────────────────────────────────────────────────────────

pub async fn create_listing(
    env: &impl Env,
    caller: Principal,
    arg: CreateListingArg,
) -> MarketResult<Listing> {
    require_caller(caller)?;
    let cfg = require_config()?;
    require_tradable(&arg.collection)?;
    if arg.price_e8s < cfg.min_price_e8s {
        return Err(MarketplaceError::PriceTooLow {
            min_price_e8s: cfg.min_price_e8s,
        });
    }
    check_existing_listing(caller, &arg)?;

    let owner = env
        .nft_owner_of(arg.collection, arg.token_id.clone())
        .await
        .map_err(MarketplaceError::CallFailed)?;
    if !owner.is_some_and(|o| o.is_default_of(&caller)) {
        return Err(MarketplaceError::NotOwner);
    }
    let approved = env
        .nft_is_approved(
            arg.collection,
            arg.token_id.clone(),
            Account::default_of(env.self_id()),
        )
        .await
        .map_err(MarketplaceError::CallFailed)?;
    if !approved {
        return Err(MarketplaceError::NotApproved);
    }

    // State may have changed while awaiting.
    check_existing_listing(caller, &arg)?;
    let now = env.now();
    if let Some(mut stale) = storage::active_listing_for(&arg.collection, &arg.token_id) {
        // Left by a previous owner; the caller owns the token now.
        stale.status = ListingStatus::Cancelled;
        stale.updated_at = now;
        storage::save_listing(&stale);
    }
    let listing = Listing {
        id: storage::next_listing_id(),
        collection: arg.collection,
        token_id: arg.token_id,
        seller: caller,
        price_e8s: arg.price_e8s,
        status: ListingStatus::Active,
        created_at: now,
        updated_at: now,
    };
    storage::save_listing(&listing);
    Ok(listing)
}

fn check_existing_listing(caller: Principal, arg: &CreateListingArg) -> MarketResult<()> {
    match storage::active_listing_for(&arg.collection, &arg.token_id) {
        Some(l) if l.status == ListingStatus::Locked => Err(MarketplaceError::PurchaseInProgress),
        Some(l) if l.seller == caller => Err(MarketplaceError::AlreadyListed { listing_id: l.id }),
        _ => Ok(()),
    }
}

pub fn cancel_listing(
    caller: Principal,
    listing_id: u64,
    is_admin: bool,
    now: u64,
) -> MarketResult<Listing> {
    let mut l = storage::get_listing(listing_id).ok_or(MarketplaceError::ListingNotFound)?;
    if !is_admin && l.seller != caller {
        return Err(MarketplaceError::Unauthorized);
    }
    match l.status {
        ListingStatus::Active => {}
        ListingStatus::Locked => return Err(MarketplaceError::PurchaseInProgress),
        _ => return Err(MarketplaceError::ListingNotActive),
    }
    l.status = ListingStatus::Cancelled;
    l.updated_at = now;
    storage::save_listing(&l);
    Ok(l)
}

pub const MAX_LISTINGS_PAGE: u64 = 100;

/// Listings on sale in tradable collections, filtered, sorted and paged.
/// Ties are broken by listing id (newer first) so pages are stable.
pub fn query_listings(q: ListingQuery) -> ListingPage {
    let mut items: Vec<Listing> = storage::active_listings(q.collection.as_ref())
        .into_iter()
        .filter(|l| {
            l.status == ListingStatus::Active
                && storage::get_collection(&l.collection).is_some_and(|c| c.status.is_tradable())
        })
        .collect();
    let newest = |a: &Listing, b: &Listing| b.created_at.cmp(&a.created_at).then(b.id.cmp(&a.id));
    match q.sort.unwrap_or_default() {
        ListingSort::Newest => items.sort_by(newest),
        ListingSort::Oldest => items.sort_by(|a, b| newest(b, a)),
        ListingSort::PriceAsc => {
            items.sort_by(|a, b| a.price_e8s.cmp(&b.price_e8s).then_with(|| newest(a, b)))
        }
        ListingSort::PriceDesc => {
            items.sort_by(|a, b| b.price_e8s.cmp(&a.price_e8s).then_with(|| newest(a, b)))
        }
    }
    let total = items.len() as u64;
    let items = items
        .into_iter()
        .skip(q.offset as usize)
        .take(q.limit.min(MAX_LISTINGS_PAGE) as usize)
        .collect();
    ListingPage { items, total }
}

// ── Purchase ───────────────────────────────────────────────────────────────

/// Buys a listing. Validation errors come back as `Err`. Once a tx exists it
/// is returned as `Ok` with its current status:
/// Completed / PayoutPending mean the buyer has the NFT.
pub async fn buy(env: &impl Env, caller: Principal, listing_id: u64) -> MarketResult<MarketTx> {
    require_caller(caller)?;
    let cfg = require_config()?;
    let mut listing = storage::get_listing(listing_id).ok_or(MarketplaceError::ListingNotFound)?;
    match listing.status {
        ListingStatus::Active => {}
        ListingStatus::Locked => return Err(MarketplaceError::PurchaseInProgress),
        _ => return Err(MarketplaceError::ListingNotActive),
    }
    if listing.seller == caller {
        return Err(MarketplaceError::CannotBuyOwnListing);
    }
    require_tradable(&listing.collection)?;

    // Lock before the first await so nobody else can buy it.
    let now = env.now();
    listing.status = ListingStatus::Locked;
    listing.updated_at = now;
    storage::save_listing(&listing);

    let mut tx = MarketTx {
        id: storage::next_tx_id(),
        listing_id,
        collection: listing.collection,
        token_id: listing.token_id.clone(),
        seller: listing.seller,
        buyer: caller,
        price_e8s: listing.price_e8s.clone(),
        fee_e8s: Nat::from(0u8),
        status: TxStatus::PaymentPending,
        needs_review: false,
        attempts: 0,
        last_error: None,
        payment_block: None,
        nft_block: None,
        settle_block: None,
        payment_ts: None,
        nft_ts: None,
        settle_ts: None,
        created_at: now,
        updated_at: now,
    };
    storage::save_tx(&tx);
    let _guard = TxGuard::acquire(tx.id).expect("fresh tx id is never in flight");

    // Pre-flight: nothing has moved yet, so any failure just ends the tx.
    let fail = |mut tx: MarketTx, listing_status, err: MarketplaceError, now| {
        tx.status = TxStatus::Failed;
        tx.last_error = Some(format!("{err:?}"));
        tx.updated_at = now;
        storage::save_tx(&tx);
        set_listing_status(listing_id, listing_status, now);
        Err(err)
    };

    match nft_available(env, &listing).await {
        Ok(true) => {}
        Ok(false) => {
            return fail(
                tx,
                ListingStatus::Cancelled,
                MarketplaceError::NftUnavailable,
                env.now(),
            )
        }
        Err(e) => {
            return fail(
                tx,
                ListingStatus::Active,
                MarketplaceError::CallFailed(e),
                env.now(),
            )
        }
    }
    let fee = match env.icp_fee(cfg.icp_ledger).await {
        Ok(f) => f,
        Err(e) => {
            return fail(
                tx,
                ListingStatus::Active,
                MarketplaceError::CallFailed(e),
                env.now(),
            )
        }
    };
    if tx.price_e8s <= fee {
        let err = MarketplaceError::PriceTooLow {
            min_price_e8s: fee.clone() + 1u8,
        };
        return fail(tx, ListingStatus::Cancelled, err, env.now());
    }

    tx.fee_e8s = fee;
    tx.payment_ts = Some(env.now());
    tx.updated_at = env.now();
    storage::save_tx(&tx);

    let tx = drive(env, tx.id, &cfg).await;
    if tx.status == TxStatus::Failed {
        return Err(MarketplaceError::PaymentFailed(
            tx.last_error.unwrap_or_default(),
        ));
    }
    Ok(tx)
}

/// The seller still owns the NFT and the oracle may still move it.
async fn nft_available(env: &impl Env, listing: &Listing) -> Result<bool, String> {
    let owner = env
        .nft_owner_of(listing.collection, listing.token_id.clone())
        .await?;
    if !owner.is_some_and(|o| o.is_default_of(&listing.seller)) {
        return Ok(false);
    }
    env.nft_is_approved(
        listing.collection,
        listing.token_id.clone(),
        Account::default_of(env.self_id()),
    )
    .await
}

enum Step {
    /// The tx moved on (or ended); run the next step now.
    Continue,
    /// Wait for the clock.
    Wait,
}

/// Runs steps until the tx ends or has to wait. Caller holds the tx guard.
async fn drive(env: &impl Env, tx_id: u64, cfg: &MarketplaceConfig) -> MarketTx {
    loop {
        let tx = storage::get_tx(tx_id).expect("driven tx exists");
        if tx.status.is_terminal() || tx.needs_review {
            return tx;
        }
        let step = match tx.status {
            TxStatus::PaymentPending => step_payment(env, tx, cfg).await,
            TxStatus::NftTransferPending => step_nft(env, tx, cfg).await,
            TxStatus::PayoutPending => step_settle(env, tx, cfg, false).await,
            TxStatus::RefundPending => step_settle(env, tx, cfg, true).await,
            _ => unreachable!("terminal statuses return above"),
        };
        if let Step::Wait = step {
            return storage::get_tx(tx_id).expect("driven tx exists");
        }
    }
}

/// Moves the tx to `status` and starts the next step with a fresh attempt
/// counter.
fn advance(env: &impl Env, tx: &mut MarketTx, status: TxStatus) {
    let now = env.now();
    tx.status = status;
    tx.attempts = 0;
    tx.updated_at = now;
    match tx.status {
        TxStatus::NftTransferPending => tx.nft_ts = Some(now),
        TxStatus::PayoutPending | TxStatus::RefundPending => tx.settle_ts = Some(now),
        _ => {}
    }
    storage::save_tx(tx);
}

/// Records a failed attempt. Flags the tx for review once retries run out.
fn record_failure(env: &impl Env, tx: &mut MarketTx, cfg: &MarketplaceConfig, err: String) -> Step {
    tx.attempts += 1;
    tx.last_error = Some(err);
    tx.updated_at = env.now();
    if tx.attempts >= cfg.max_retries {
        tx.needs_review = true;
    }
    storage::save_tx(tx);
    Step::Wait
}

async fn step_payment(env: &impl Env, mut tx: MarketTx, cfg: &MarketplaceConfig) -> Step {
    let Some(ts) = tx.payment_ts else {
        // Stopped before the first payment attempt: nothing moved.
        tx.status = TxStatus::Failed;
        tx.last_error = Some("interrupted before payment".into());
        tx.updated_at = env.now();
        storage::save_tx(&tx);
        set_listing_status(tx.listing_id, ListingStatus::Active, env.now());
        return Step::Continue;
    };
    let outcome = env
        .icp_transfer_from(
            cfg.icp_ledger,
            IcpTransferFrom {
                from: Account::default_of(tx.buyer),
                to: escrow(env),
                amount: tx.price_e8s.clone(),
                fee: tx.fee_e8s.clone(),
                memo: memo(MemoStep::Payment, tx.id),
                created_at_time: ts,
            },
        )
        .await;
    match outcome {
        TransferOutcome::Ok(block) => {
            tx.payment_block = Some(block);
            advance(env, &mut tx, TxStatus::NftTransferPending);
            Step::Continue
        }
        TransferOutcome::Rejected(e) => end_unpaid(env, tx, e),
        TransferOutcome::BadFee(f) => end_unpaid(env, tx, format!("BadFee, expected {f}")),
        TransferOutcome::Unknown(e) => record_failure(env, &mut tx, cfg, e),
    }
}

/// Payment was refused: no funds moved, the listing goes back on sale.
fn end_unpaid(env: &impl Env, mut tx: MarketTx, err: String) -> Step {
    tx.status = TxStatus::Failed;
    tx.last_error = Some(err);
    tx.updated_at = env.now();
    storage::save_tx(&tx);
    set_listing_status(tx.listing_id, ListingStatus::Active, env.now());
    Step::Continue
}

async fn step_nft(env: &impl Env, mut tx: MarketTx, cfg: &MarketplaceConfig) -> Step {
    let ts = tx.nft_ts.unwrap_or_else(|| env.now());
    tx.nft_ts = Some(ts);
    let outcome = env
        .nft_transfer_from(
            tx.collection,
            NftTransferFrom {
                from: Account::default_of(tx.seller),
                to: Account::default_of(tx.buyer),
                token_id: tx.token_id.clone(),
                memo: memo(MemoStep::Nft, tx.id),
                created_at_time: ts,
            },
        )
        .await;
    if let TransferOutcome::Ok(block) = outcome {
        tx.nft_block = Some(block);
        return nft_delivered(env, tx);
    }

    // Not confirmed: ask the collection who owns the token now.
    let error = format!("{outcome:?}");
    match env.nft_owner_of(tx.collection, tx.token_id.clone()).await {
        Ok(Some(owner)) if owner.is_default_of(&tx.buyer) => nft_delivered(env, tx),
        Ok(Some(owner)) if owner.is_default_of(&tx.seller) => match outcome {
            TransferOutcome::Unknown(_) if tx.attempts + 1 < cfg.max_retries => {
                // The seller still has it, so the attempt did not execute and
                // the next one may use a fresh timestamp (avoids TooOld).
                tx.nft_ts = None;
                record_failure(env, &mut tx, cfg, error)
            }
            // Refused, or retries ran out: give the money back.
            _ => start_refund(env, tx, error),
        },
        // Token gone or moved elsewhere: it can no longer be delivered.
        Ok(_) => start_refund(env, tx, error),
        Err(e) => record_failure(env, &mut tx, cfg, format!("{error}; owner_of: {e}")),
    }
}

fn nft_delivered(env: &impl Env, mut tx: MarketTx) -> Step {
    set_listing_status(tx.listing_id, ListingStatus::Sold, env.now());
    advance(env, &mut tx, TxStatus::PayoutPending);
    Step::Continue
}

fn start_refund(env: &impl Env, mut tx: MarketTx, err: String) -> Step {
    tx.last_error = Some(err);
    set_listing_status(tx.listing_id, ListingStatus::Cancelled, env.now());
    advance(env, &mut tx, TxStatus::RefundPending);
    Step::Continue
}

/// Payout to the seller or refund to the buyer. Both get `price - fee`:
/// the escrow pays the outgoing ledger fee.
async fn step_settle(
    env: &impl Env,
    mut tx: MarketTx,
    cfg: &MarketplaceConfig,
    refund: bool,
) -> Step {
    let ts = tx.settle_ts.unwrap_or_else(|| env.now());
    tx.settle_ts = Some(ts);
    let (to, step) = if refund {
        (tx.buyer, MemoStep::Refund)
    } else {
        (tx.seller, MemoStep::Payout)
    };
    let amount = if tx.price_e8s > tx.fee_e8s {
        tx.price_e8s.clone() - tx.fee_e8s.clone()
    } else {
        Nat::from(0u8)
    };
    let outcome = env
        .icp_transfer(
            cfg.icp_ledger,
            IcpTransfer {
                from_subaccount: Some(ESCROW_SUBACCOUNT.to_vec()),
                to: Account::default_of(to),
                amount,
                fee: tx.fee_e8s.clone(),
                memo: memo(step, tx.id),
                created_at_time: ts,
            },
        )
        .await;
    match outcome {
        TransferOutcome::Ok(block) => {
            tx.settle_block = Some(block);
            tx.status = if refund {
                TxStatus::Refunded
            } else {
                TxStatus::Completed
            };
            tx.attempts = 0;
            tx.updated_at = env.now();
            storage::save_tx(&tx);
            Step::Continue
        }
        TransferOutcome::BadFee(expected) => {
            // Not executed. New args, so a new dedup timestamp is safe.
            tx.fee_e8s = expected.clone();
            tx.settle_ts = Some(env.now());
            let step = record_failure(env, &mut tx, cfg, format!("BadFee, expected {expected}"));
            if tx.needs_review {
                step
            } else {
                Step::Continue
            }
        }
        TransferOutcome::Rejected(e) | TransferOutcome::Unknown(e) => {
            record_failure(env, &mut tx, cfg, e)
        }
    }
}

// ── Clock & admin ──────────────────────────────────────────────────────────

/// Retries pending txs. Returns how many were processed.
pub async fn run_clock(env: &impl Env) -> u32 {
    let Some(_clock) = ClockGuard::acquire() else {
        return 0;
    };
    let Some(cfg) = storage::config() else {
        return 0;
    };
    let mut processed = 0;
    for id in storage::pending_tx_ids() {
        if processed as usize >= CLOCK_BATCH {
            break;
        }
        let Some(tx) = storage::get_tx(id) else {
            continue;
        };
        if tx.needs_review {
            continue;
        }
        let Some(_guard) = TxGuard::acquire(id) else {
            continue; // being driven by a buy call right now
        };
        drive(env, id, &cfg).await;
        processed += 1;
    }
    processed
}

/// Clears the review flag and drives the tx again. `reset_dedup` gives the
/// current step a new `created_at_time`; use it only after checking on the
/// ledger that the previous attempt did not execute (e.g. after TooOld).
pub async fn admin_retry_tx(
    env: &impl Env,
    tx_id: u64,
    reset_dedup: bool,
) -> MarketResult<MarketTx> {
    let cfg = require_config()?;
    let mut tx = storage::get_tx(tx_id).ok_or(MarketplaceError::TxNotFound)?;
    if tx.status.is_terminal() {
        return Err(MarketplaceError::InvalidTxState);
    }
    let _guard = TxGuard::acquire(tx_id).ok_or(MarketplaceError::PurchaseInProgress)?;
    tx.needs_review = false;
    tx.attempts = 0;
    if reset_dedup {
        let now = env.now();
        match tx.status {
            TxStatus::PaymentPending => tx.payment_ts = Some(now),
            TxStatus::NftTransferPending => tx.nft_ts = Some(now),
            _ => tx.settle_ts = Some(now),
        }
    }
    tx.updated_at = env.now();
    storage::save_tx(&tx);
    Ok(drive(env, tx_id, &cfg).await)
}

/// Ends a stuck tx after it was settled by hand.
pub fn admin_close_tx(tx_id: u64, note: String, now: u64) -> MarketResult<MarketTx> {
    let mut tx = storage::get_tx(tx_id).ok_or(MarketplaceError::TxNotFound)?;
    if tx.status.is_terminal() {
        return Err(MarketplaceError::InvalidTxState);
    }
    if TxGuard::acquire(tx_id).is_none() {
        return Err(MarketplaceError::PurchaseInProgress);
    }
    if let Some(l) = storage::get_listing(tx.listing_id) {
        if l.status == ListingStatus::Locked {
            set_listing_status(l.id, ListingStatus::Cancelled, now);
        }
    }
    tx.last_error = Some(format!("closed by admin from {:?}: {note}", tx.status));
    tx.status = TxStatus::ClosedByAdmin;
    tx.updated_at = now;
    storage::save_tx(&tx);
    Ok(tx)
}
