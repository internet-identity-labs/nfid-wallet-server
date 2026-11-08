use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};

use candid::{Nat, Principal};

use super::env::*;
use super::service::{self, ESCROW_SUBACCOUNT};
use super::storage;
use super::types::*;

/// Mock futures never suspend, so one poll is enough.
fn run<F: Future>(f: F) -> F::Output {
    let mut cx = Context::from_waker(Waker::noop());
    match pin!(f).poll(&mut cx) {
        Poll::Ready(v) => v,
        Poll::Pending => panic!("mock future suspended"),
    }
}

fn p(n: u8) -> Principal {
    Principal::from_slice(&[n; 29])
}

fn oracle() -> Principal {
    p(1)
}
fn collection() -> Principal {
    p(2)
}
fn seller() -> Principal {
    p(3)
}
fn buyer() -> Principal {
    p(4)
}
fn ledger() -> Principal {
    p(5)
}

const PRICE: u64 = 100_000_000;
const FEE: u64 = 10_000;

#[derive(Debug, Clone, PartialEq)]
struct Settle {
    to: Account,
    amount: Nat,
    created_at_time: u64,
}

struct MockEnv {
    now: Cell<u64>,
    owner: RefCell<Option<Account>>,
    approved: Cell<bool>,
    owner_call_fails: Cell<bool>,
    payment: RefCell<VecDeque<TransferOutcome>>,
    nft: RefCell<VecDeque<TransferOutcome>>,
    settle: RefCell<VecDeque<TransferOutcome>>,
    payment_ts: RefCell<Vec<u64>>,
    settles: RefCell<Vec<Settle>>,
}

impl MockEnv {
    fn new() -> Self {
        Self {
            now: Cell::new(1_000),
            owner: RefCell::new(Some(Account::default_of(seller()))),
            approved: Cell::new(true),
            owner_call_fails: Cell::new(false),
            payment: RefCell::default(),
            nft: RefCell::default(),
            settle: RefCell::default(),
            payment_ts: RefCell::default(),
            settles: RefCell::default(),
        }
    }

    fn tick(&self) -> u64 {
        let t = self.now.get() + 1;
        self.now.set(t);
        t
    }
}

fn ok() -> TransferOutcome {
    TransferOutcome::Ok(Nat::from(7u8))
}

impl Env for MockEnv {
    fn now(&self) -> u64 {
        self.tick()
    }

    fn self_id(&self) -> Principal {
        oracle()
    }

    async fn icp_fee(&self, _: Principal) -> Result<Nat, String> {
        Ok(Nat::from(FEE))
    }

    async fn icp_transfer_from(&self, l: Principal, a: IcpTransferFrom) -> TransferOutcome {
        assert_eq!(l, ledger());
        assert_eq!(a.to.subaccount, Some(ESCROW_SUBACCOUNT.to_vec()));
        assert_eq!(a.amount, Nat::from(PRICE));
        self.payment_ts.borrow_mut().push(a.created_at_time);
        self.payment.borrow_mut().pop_front().unwrap_or_else(ok)
    }

    async fn icp_transfer(&self, _: Principal, a: IcpTransfer) -> TransferOutcome {
        assert_eq!(a.from_subaccount, Some(ESCROW_SUBACCOUNT.to_vec()));
        self.settles.borrow_mut().push(Settle {
            to: a.to,
            amount: a.amount,
            created_at_time: a.created_at_time,
        });
        self.settle.borrow_mut().pop_front().unwrap_or_else(ok)
    }

    async fn nft_owner_of(&self, _: Principal, _: Nat) -> Result<Option<Account>, String> {
        if self.owner_call_fails.get() {
            return Err("unreachable".into());
        }
        Ok(self.owner.borrow().clone())
    }

    async fn nft_is_approved(
        &self,
        _: Principal,
        _: Nat,
        spender: Account,
    ) -> Result<bool, String> {
        assert_eq!(spender, Account::default_of(oracle()));
        Ok(self.approved.get())
    }

    async fn nft_transfer_from(&self, _: Principal, a: NftTransferFrom) -> TransferOutcome {
        let out = self.nft.borrow_mut().pop_front().unwrap_or_else(ok);
        if matches!(out, TransferOutcome::Ok(_)) {
            *self.owner.borrow_mut() = Some(a.to);
        }
        out
    }
}

fn setup(max_retries: u32) {
    storage::reset_for_tests();
    storage::CONFIG.with(|c| {
        *c.borrow_mut() = Some(MarketplaceConfig {
            icp_ledger: ledger(),
            min_price_e8s: Nat::from(1_000_000u64),
            max_retries,
        })
    });
    service::store_collection(
        NftCollectionRequest {
            canister_id: collection(),
            name: "Test".into(),
            symbol: "TST".into(),
            logo: None,
            description: None,
        },
        0,
    );
}

fn list(env: &MockEnv) -> Listing {
    run(service::create_listing(
        env,
        seller(),
        CreateListingArg {
            collection: collection(),
            token_id: Nat::from(1u8),
            price_e8s: Nat::from(PRICE),
        },
    ))
    .expect("listing created")
}

fn set_status(status: NftCollectionStatus) {
    storage::COLLECTIONS.with(|c| c.borrow_mut().get_mut(&collection()).unwrap().status = status);
}

// ── Listings ───────────────────────────────────────────────────────────────

#[test]
fn create_listing_checks_owner_approval_price_and_collection() {
    setup(3);
    let env = MockEnv::new();
    let arg = |price: u64| CreateListingArg {
        collection: collection(),
        token_id: Nat::from(1u8),
        price_e8s: Nat::from(price),
    };

    let r = run(service::create_listing(&env, buyer(), arg(PRICE)));
    assert_eq!(r.unwrap_err(), MarketplaceError::NotOwner);

    let r = run(service::create_listing(&env, seller(), arg(10)));
    assert!(matches!(r, Err(MarketplaceError::PriceTooLow { .. })));

    env.approved.set(false);
    let r = run(service::create_listing(&env, seller(), arg(PRICE)));
    assert_eq!(r.unwrap_err(), MarketplaceError::NotApproved);
    env.approved.set(true);

    set_status(NftCollectionStatus::Spam);
    let r = run(service::create_listing(&env, seller(), arg(PRICE)));
    assert_eq!(r.unwrap_err(), MarketplaceError::CollectionNotAllowed);
    set_status(NftCollectionStatus::Community);

    let l = list(&env);
    let r = run(service::create_listing(&env, seller(), arg(PRICE)));
    assert_eq!(
        r.unwrap_err(),
        MarketplaceError::AlreadyListed { listing_id: l.id }
    );
}

#[test]
fn new_owner_listing_replaces_stale_listing() {
    setup(3);
    let env = MockEnv::new();
    let old = list(&env);
    *env.owner.borrow_mut() = Some(Account::default_of(buyer()));

    let new = run(service::create_listing(
        &env,
        buyer(),
        CreateListingArg {
            collection: collection(),
            token_id: Nat::from(1u8),
            price_e8s: Nat::from(PRICE),
        },
    ))
    .unwrap();

    assert_eq!(
        storage::get_listing(old.id).unwrap().status,
        ListingStatus::Cancelled
    );
    assert_eq!(
        storage::active_listing_for(&collection(), &Nat::from(1u8))
            .unwrap()
            .id,
        new.id
    );
}

#[test]
fn only_seller_or_admin_cancels() {
    setup(3);
    let env = MockEnv::new();
    let l = list(&env);
    assert_eq!(
        service::cancel_listing(buyer(), l.id, false, 0).unwrap_err(),
        MarketplaceError::Unauthorized
    );
    let c = service::cancel_listing(buyer(), l.id, true, 0).unwrap();
    assert_eq!(c.status, ListingStatus::Cancelled);
    assert!(storage::active_listing_for(&collection(), &Nat::from(1u8)).is_none());
}

// ── Purchase ───────────────────────────────────────────────────────────────

#[test]
fn buy_happy_path_pays_seller_price_minus_fee() {
    setup(3);
    let env = MockEnv::new();
    let l = list(&env);

    let tx = run(service::buy(&env, buyer(), l.id)).unwrap();

    assert_eq!(tx.status, TxStatus::Completed);
    assert_eq!(tx.fee_e8s, Nat::from(FEE));
    assert!(tx.payment_block.is_some() && tx.nft_block.is_some() && tx.settle_block.is_some());
    assert_eq!(*env.owner.borrow(), Some(Account::default_of(buyer())));
    assert_eq!(
        env.settles.borrow()[0],
        Settle {
            to: Account::default_of(seller()),
            amount: Nat::from(PRICE - FEE),
            created_at_time: tx.settle_ts.unwrap(),
        }
    );
    assert_eq!(
        storage::get_listing(l.id).unwrap().status,
        ListingStatus::Sold
    );
    assert!(storage::pending_tx_ids().is_empty());
}

#[test]
fn buy_rejects_own_listing_and_locked_listing() {
    setup(3);
    let env = MockEnv::new();
    let l = list(&env);
    assert_eq!(
        run(service::buy(&env, seller(), l.id)).unwrap_err(),
        MarketplaceError::CannotBuyOwnListing
    );

    // Payment outcome unknown: the listing stays locked.
    env.payment
        .borrow_mut()
        .push_back(TransferOutcome::Unknown("timeout".into()));
    let tx = run(service::buy(&env, buyer(), l.id)).unwrap();
    assert_eq!(tx.status, TxStatus::PaymentPending);
    assert_eq!(
        run(service::buy(&env, p(9), l.id)).unwrap_err(),
        MarketplaceError::PurchaseInProgress
    );
    assert_eq!(
        service::cancel_listing(seller(), l.id, false, 0).unwrap_err(),
        MarketplaceError::PurchaseInProgress
    );
}

#[test]
fn buy_fails_without_moving_funds_when_nft_unavailable() {
    setup(3);
    let env = MockEnv::new();
    let l = list(&env);
    env.approved.set(false); // seller revoked approval

    let r = run(service::buy(&env, buyer(), l.id));

    assert_eq!(r.unwrap_err(), MarketplaceError::NftUnavailable);
    assert!(env.payment_ts.borrow().is_empty());
    assert_eq!(
        storage::get_listing(l.id).unwrap().status,
        ListingStatus::Cancelled
    );
    assert_eq!(storage::get_tx(0).unwrap().status, TxStatus::Failed);
}

#[test]
fn rejected_payment_reopens_listing() {
    setup(3);
    let env = MockEnv::new();
    let l = list(&env);
    env.payment
        .borrow_mut()
        .push_back(TransferOutcome::Rejected("InsufficientAllowance".into()));

    let r = run(service::buy(&env, buyer(), l.id));

    assert!(matches!(r, Err(MarketplaceError::PaymentFailed(_))));
    assert_eq!(
        storage::get_listing(l.id).unwrap().status,
        ListingStatus::Active
    );
    assert!(storage::pending_tx_ids().is_empty());
}

#[test]
fn clock_retries_unknown_payment_with_same_timestamp() {
    setup(3);
    let env = MockEnv::new();
    let l = list(&env);
    env.payment
        .borrow_mut()
        .push_back(TransferOutcome::Unknown("timeout".into()));
    run(service::buy(&env, buyer(), l.id)).unwrap();

    // The ledger did execute the first call: the retry is a duplicate.
    env.payment
        .borrow_mut()
        .push_back(TransferOutcome::Ok(Nat::from(42u8)));
    assert_eq!(run(service::run_clock(&env)), 1);

    let tx = storage::get_tx(0).unwrap();
    assert_eq!(tx.status, TxStatus::Completed);
    assert_eq!(tx.payment_block, Some(Nat::from(42u8)));
    let ts = env.payment_ts.borrow();
    assert_eq!(ts.len(), 2);
    assert_eq!(ts[0], ts[1]);
}

#[test]
fn rejected_nft_transfer_refunds_buyer() {
    setup(3);
    let env = MockEnv::new();
    let l = list(&env);
    env.nft
        .borrow_mut()
        .push_back(TransferOutcome::Rejected("Unauthorized".into()));

    let tx = run(service::buy(&env, buyer(), l.id)).unwrap();

    assert_eq!(tx.status, TxStatus::Refunded);
    assert_eq!(env.settles.borrow()[0].to, Account::default_of(buyer()));
    assert_eq!(env.settles.borrow()[0].amount, Nat::from(PRICE - FEE));
    assert_eq!(
        storage::get_listing(l.id).unwrap().status,
        ListingStatus::Cancelled
    );
}

#[test]
fn unknown_nft_transfer_resolved_by_owner_check() {
    setup(3);
    let env = MockEnv::new();
    let l = list(&env);
    env.nft
        .borrow_mut()
        .push_back(TransferOutcome::Unknown("reply lost".into()));
    let tx = run(service::buy(&env, buyer(), l.id)).unwrap();
    // The seller still owns it, so the tx waits for the clock.
    assert_eq!(tx.status, TxStatus::NftTransferPending);
    assert_eq!(tx.attempts, 1);

    // Next attempt executes but the reply is lost; owner_of shows the buyer.
    *env.owner.borrow_mut() = Some(Account::default_of(buyer()));
    env.nft
        .borrow_mut()
        .push_back(TransferOutcome::Unknown("reply lost".into()));
    run(service::run_clock(&env));

    let tx = storage::get_tx(tx.id).unwrap();
    assert_eq!(tx.status, TxStatus::Completed);
    assert_eq!(env.settles.borrow()[0].to, Account::default_of(seller()));
}

#[test]
fn nft_retries_exhausted_falls_back_to_refund() {
    setup(2);
    let env = MockEnv::new();
    let l = list(&env);
    for _ in 0..2 {
        env.nft
            .borrow_mut()
            .push_back(TransferOutcome::Unknown("down".into()));
    }
    let tx = run(service::buy(&env, buyer(), l.id)).unwrap();
    assert_eq!(tx.status, TxStatus::NftTransferPending);

    run(service::run_clock(&env));

    assert_eq!(storage::get_tx(tx.id).unwrap().status, TxStatus::Refunded);
}

#[test]
fn failing_payout_is_flagged_for_review_then_admin_retry_completes() {
    setup(2);
    let env = MockEnv::new();
    let l = list(&env);
    for _ in 0..2 {
        env.settle
            .borrow_mut()
            .push_back(TransferOutcome::Unknown("ledger down".into()));
    }

    let tx = run(service::buy(&env, buyer(), l.id)).unwrap();
    assert_eq!(tx.status, TxStatus::PayoutPending);
    run(service::run_clock(&env));
    let tx = storage::get_tx(tx.id).unwrap();
    assert!(tx.needs_review);
    assert_eq!(tx.attempts, 2);

    // The clock leaves flagged txs alone.
    assert_eq!(run(service::run_clock(&env)), 0);

    let tx = run(service::admin_retry_tx(&env, tx.id, false)).unwrap();
    assert_eq!(tx.status, TxStatus::Completed);
    assert!(!tx.needs_review);
    let settles = env.settles.borrow();
    assert_eq!(settles.len(), 3);
    assert!(settles
        .iter()
        .all(|s| s.created_at_time == settles[0].created_at_time));
}

#[test]
fn bad_fee_on_payout_retries_with_expected_fee() {
    setup(3);
    let env = MockEnv::new();
    let l = list(&env);
    env.settle
        .borrow_mut()
        .push_back(TransferOutcome::BadFee(Nat::from(20_000u64)));

    let tx = run(service::buy(&env, buyer(), l.id)).unwrap();

    assert_eq!(tx.status, TxStatus::Completed);
    assert_eq!(tx.fee_e8s, Nat::from(20_000u64));
    assert_eq!(env.settles.borrow()[1].amount, Nat::from(PRICE - 20_000));
}

#[test]
fn spam_collection_blocks_purchase() {
    setup(3);
    let env = MockEnv::new();
    let l = list(&env);
    set_status(NftCollectionStatus::Spam);
    assert_eq!(
        run(service::buy(&env, buyer(), l.id)).unwrap_err(),
        MarketplaceError::CollectionNotAllowed
    );
}

#[test]
fn owner_check_failure_keeps_tx_pending() {
    setup(3);
    let env = MockEnv::new();
    let l = list(&env);
    env.nft
        .borrow_mut()
        .push_back(TransferOutcome::Unknown("down".into()));
    // Availability check passes, then the collection stops answering.
    let tx = run(service::buy(&env, buyer(), l.id));
    assert_eq!(tx.unwrap().status, TxStatus::NftTransferPending);
    env.owner_call_fails.set(true);
    env.nft
        .borrow_mut()
        .push_back(TransferOutcome::Unknown("down".into()));
    run(service::run_clock(&env));
    let tx = storage::get_tx(0).unwrap();
    assert_eq!(tx.status, TxStatus::NftTransferPending);
    assert_eq!(tx.attempts, 2);
    assert!(tx.last_error.unwrap().contains("owner_of"));
}

#[test]
fn admin_close_ends_stuck_tx() {
    setup(1);
    let env = MockEnv::new();
    let l = list(&env);
    env.payment
        .borrow_mut()
        .push_back(TransferOutcome::Unknown("down".into()));

    let tx = run(service::buy(&env, buyer(), l.id)).unwrap();
    assert!(tx.needs_review);

    let tx = service::admin_close_tx(tx.id, "refunded manually".into(), 5).unwrap();
    assert_eq!(tx.status, TxStatus::ClosedByAdmin);
    assert_eq!(
        storage::get_listing(l.id).unwrap().status,
        ListingStatus::Cancelled
    );
    assert!(storage::pending_tx_ids().is_empty());
}

#[test]
fn store_collection_does_not_overwrite_existing() {
    setup(3);
    set_status(NftCollectionStatus::Verified);
    service::store_collection(
        NftCollectionRequest {
            canister_id: collection(),
            name: "Fake".into(),
            symbol: "F".into(),
            logo: None,
            description: None,
        },
        9,
    );
    let c = storage::get_collection(&collection()).unwrap();
    assert_eq!(c.name, "Test");
    assert_eq!(c.status, NftCollectionStatus::Verified);
}

#[test]
fn upgrade_blob_roundtrip() {
    storage::write_upgrade_blob(b"hello");
    assert_eq!(storage::read_upgrade_blob(), b"hello");
    storage::write_upgrade_blob(&vec![7u8; 200_000]);
    assert_eq!(storage::read_upgrade_blob(), vec![7u8; 200_000]);
}

// ── Listing queries ────────────────────────────────────────────────────────

fn put_listing(id: u64, collection: Principal, price: u64, created_at: u64, status: ListingStatus) {
    storage::save_listing(&Listing {
        id,
        collection,
        token_id: Nat::from(id),
        seller: seller(),
        price_e8s: Nat::from(price),
        status,
        created_at,
        updated_at: created_at,
    });
}

fn query(
    collection: Option<Principal>,
    sort: Option<ListingSort>,
    offset: u64,
    limit: u64,
) -> (Vec<u64>, u64) {
    let page = service::query_listings(ListingQuery {
        collection,
        sort,
        offset,
        limit,
    });
    (page.items.iter().map(|l| l.id).collect(), page.total)
}

#[test]
fn get_listings_sorts_filters_and_pages() {
    setup(3);
    let spam = p(8);
    service::store_collection(
        NftCollectionRequest {
            canister_id: spam,
            name: "Spam".into(),
            symbol: "S".into(),
            logo: None,
            description: None,
        },
        0,
    );
    storage::COLLECTIONS
        .with(|c| c.borrow_mut().get_mut(&spam).unwrap().status = NftCollectionStatus::Spam);
    let other = p(9);
    service::store_collection(
        NftCollectionRequest {
            canister_id: other,
            name: "Other".into(),
            symbol: "O".into(),
            logo: None,
            description: None,
        },
        0,
    );

    put_listing(0, collection(), 300, 10, ListingStatus::Active);
    put_listing(1, collection(), 100, 30, ListingStatus::Active);
    put_listing(2, other, 200, 20, ListingStatus::Active);
    put_listing(3, collection(), 100, 40, ListingStatus::Active); // same price as 1, newer
    put_listing(4, spam, 50, 50, ListingStatus::Active); // not tradable
    put_listing(5, collection(), 10, 60, ListingStatus::Locked); // purchase in progress
    put_listing(6, collection(), 10, 70, ListingStatus::Cancelled);
    put_listing(10, collection(), 400, 5, ListingStatus::Active); // token "10" vs "2" ordering

    assert_eq!(query(None, None, 0, 10), (vec![3, 1, 2, 0, 10], 5));
    assert_eq!(
        query(None, Some(ListingSort::Oldest), 0, 10).0,
        vec![10, 0, 2, 1, 3]
    );
    assert_eq!(
        query(None, Some(ListingSort::PriceAsc), 0, 10).0,
        vec![3, 1, 2, 0, 10]
    );
    assert_eq!(
        query(None, Some(ListingSort::PriceDesc), 0, 10).0,
        vec![10, 0, 2, 3, 1]
    );

    assert_eq!(
        query(Some(collection()), Some(ListingSort::PriceAsc), 0, 10),
        (vec![3, 1, 0, 10], 4)
    );
    assert_eq!(query(Some(spam), None, 0, 10), (vec![], 0));

    assert_eq!(
        query(None, Some(ListingSort::PriceAsc), 1, 2),
        (vec![1, 2], 5)
    );
    assert_eq!(query(None, None, 10, 10), (vec![], 5));
}

#[test]
fn get_listings_caps_limit() {
    setup(3);
    for i in 0..150 {
        put_listing(i, collection(), 1_000_000, i, ListingStatus::Active);
    }
    let (items, total) = query(None, None, 0, 1_000);
    assert_eq!(total, 150);
    assert_eq!(items.len() as u64, service::MAX_LISTINGS_PAGE);
}
