use std::borrow::Cow;

use candid::{CandidType, Decode, Deserialize, Encode, Nat, Principal};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::Storable;
use serde::Serialize;

/// ICRC account (shared by ICRC-1/2 and ICRC-7/37).
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Account {
    pub owner: Principal,
    pub subaccount: Option<Vec<u8>>,
}

impl Account {
    pub fn default_of(owner: Principal) -> Self {
        Self {
            owner,
            subaccount: None,
        }
    }

    /// Default account of `principal`: no subaccount or an all-zero one.
    pub fn is_default_of(&self, principal: &Principal) -> bool {
        self.owner == *principal
            && self
                .subaccount
                .as_ref()
                .is_none_or(|s| s.iter().all(|b| *b == 0))
    }
}

// ── Collections ────────────────────────────────────────────────────────────

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum NftCollectionStatus {
    Verified,
    Community,
    Spam,
    Hidden,
}

impl NftCollectionStatus {
    /// Collections whose NFTs can be listed and bought.
    pub fn is_tradable(&self) -> bool {
        matches!(self, Self::Verified | Self::Community)
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct NftCollection {
    pub canister_id: Principal,
    pub name: String,
    pub symbol: String,
    pub logo: Option<String>,
    pub description: Option<String>,
    pub status: NftCollectionStatus,
    pub date_added: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct NftCollectionRequest {
    pub canister_id: Principal,
    pub name: String,
    pub symbol: String,
    pub logo: Option<String>,
    pub description: Option<String>,
}

// ── Config ─────────────────────────────────────────────────────────────────

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct MarketplaceConfig {
    /// ICP ledger (must support ICRC-2).
    pub icp_ledger: Principal,
    /// Lowest allowed listing price. Must cover the ledger fee.
    pub min_price_e8s: Nat,
    /// Failed clock attempts per step before a tx is flagged for manual review.
    pub max_retries: u32,
}

// ── Listings ───────────────────────────────────────────────────────────────

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum ListingStatus {
    Active,
    /// A purchase is in progress.
    Locked,
    Sold,
    Cancelled,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct Listing {
    pub id: u64,
    pub collection: Principal,
    pub token_id: Nat,
    pub seller: Principal,
    pub price_e8s: Nat,
    pub status: ListingStatus,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(CandidType, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListingSort {
    PriceAsc,
    PriceDesc,
    #[default]
    Newest,
    Oldest,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ListingQuery {
    /// Only this collection.
    pub collection: Option<Principal>,
    /// Defaults to Newest.
    pub sort: Option<ListingSort>,
    pub offset: u64,
    /// Capped at `MAX_LISTINGS_PAGE`.
    pub limit: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ListingPage {
    pub items: Vec<Listing>,
    /// Listings matching the query, before paging.
    pub total: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CreateListingArg {
    pub collection: Principal,
    pub token_id: Nat,
    pub price_e8s: Nat,
}

// ── Transactions ───────────────────────────────────────────────────────────

/// Pending states name the step that runs next. The rest are terminal.
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum TxStatus {
    /// Pull ICP from the buyer into escrow.
    PaymentPending,
    /// Move the NFT from the seller to the buyer.
    NftTransferPending,
    /// Pay the seller from escrow.
    PayoutPending,
    /// Return ICP from escrow to the buyer.
    RefundPending,
    Completed,
    Refunded,
    /// Ended before any ICP moved.
    Failed,
    ClosedByAdmin,
}

impl TxStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Refunded | Self::Failed | Self::ClosedByAdmin
        )
    }
}

/// Purchase record. Holds only what is needed to resume or debug a purchase.
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct MarketTx {
    pub id: u64,
    pub listing_id: u64,
    pub collection: Principal,
    pub token_id: Nat,
    pub seller: Principal,
    pub buyer: Principal,
    pub price_e8s: Nat,
    pub fee_e8s: Nat,
    pub status: TxStatus,
    /// Set when retries ran out; the clock skips the tx until an admin acts.
    pub needs_review: bool,
    /// Failed attempts of the current step.
    pub attempts: u32,
    pub last_error: Option<String>,
    pub payment_block: Option<Nat>,
    pub nft_block: Option<Nat>,
    /// Block of the payout (Completed) or refund (Refunded).
    pub settle_block: Option<Nat>,
    /// `created_at_time` of each transfer. Reused on retries so the ledger
    /// rejects a repeat as a duplicate instead of moving funds twice.
    pub payment_ts: Option<u64>,
    pub nft_ts: Option<u64>,
    pub settle_ts: Option<u64>,
    pub created_at: u64,
    pub updated_at: u64,
}

// ── Errors ─────────────────────────────────────────────────────────────────

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum MarketplaceError {
    NotConfigured,
    Unauthorized,
    AnonymousCaller,
    CollectionNotFound,
    CollectionNotAllowed,
    PriceTooLow { min_price_e8s: Nat },
    NotOwner,
    NotApproved,
    AlreadyListed { listing_id: u64 },
    ListingNotFound,
    ListingNotActive,
    PurchaseInProgress,
    CannotBuyOwnListing,
    NftUnavailable,
    PaymentFailed(String),
    CallFailed(String),
    TxNotFound,
    InvalidTxState,
}

pub type MarketResult<T> = Result<T, MarketplaceError>;

// ── Stable storage encoding ────────────────────────────────────────────────

macro_rules! candid_storable {
    ($t:ty) => {
        impl Storable for $t {
            fn to_bytes(&self) -> Cow<'_, [u8]> {
                Cow::Owned(Encode!(self).expect(concat!("encode ", stringify!($t))))
            }

            fn from_bytes(bytes: Cow<[u8]>) -> Self {
                Decode!(bytes.as_ref(), $t).expect(concat!("decode ", stringify!($t)))
            }

            const BOUND: Bound = Bound::Unbounded;
        }
    };
}

candid_storable!(Listing);
candid_storable!(MarketTx);
