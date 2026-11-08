//! Marketplace state.
//!
//! Listings and transactions grow without bound, so they live in stable
//! structures and are not re-serialized on upgrade. Small state (collections,
//! config) stays on the heap and goes through the oracle's upgrade blob.

use std::cell::RefCell;
use std::collections::{BTreeMap, HashSet};

use candid::{Nat, Principal};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, Memory as _, StableBTreeMap};

use super::types::{Listing, MarketTx, MarketplaceConfig, NftCollection};

pub type VMem = VirtualMemory<DefaultMemoryImpl>;

const UPGRADE_BLOB_MEM: MemoryId = MemoryId::new(0);
const LISTINGS_MEM: MemoryId = MemoryId::new(1);
const TXS_MEM: MemoryId = MemoryId::new(2);
const ACTIVE_LISTINGS_MEM: MemoryId = MemoryId::new(3);
const PENDING_TXS_MEM: MemoryId = MemoryId::new(4);

const WASM_PAGE: u64 = 64 * 1024;

thread_local! {
    // Initialized lazily on first use. `post_upgrade` must read a legacy
    // (pre-MemoryManager) blob before anything touches this.
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static LISTINGS: RefCell<StableBTreeMap<u64, Listing, VMem>> =
        RefCell::new(StableBTreeMap::init(mem(LISTINGS_MEM)));
    static TXS: RefCell<StableBTreeMap<u64, MarketTx, VMem>> =
        RefCell::new(StableBTreeMap::init(mem(TXS_MEM)));
    // "<collection>:<token_id>" -> id of the Active or Locked listing.
    static ACTIVE_LISTINGS: RefCell<StableBTreeMap<String, u64, VMem>> =
        RefCell::new(StableBTreeMap::init(mem(ACTIVE_LISTINGS_MEM)));
    // Ids of txs that are not terminal yet.
    static PENDING_TXS: RefCell<StableBTreeMap<u64, (), VMem>> =
        RefCell::new(StableBTreeMap::init(mem(PENDING_TXS_MEM)));

    pub static COLLECTIONS: RefCell<BTreeMap<Principal, NftCollection>> =
        const { RefCell::new(BTreeMap::new()) };
    pub static CONFIG: RefCell<Option<MarketplaceConfig>> = const { RefCell::new(None) };

    // Txs currently being driven by some message; heap only.
    static IN_FLIGHT: RefCell<HashSet<u64>> = RefCell::new(HashSet::new());
    static CLOCK_RUNNING: RefCell<bool> = const { RefCell::new(false) };
}

fn mem(id: MemoryId) -> VMem {
    MEMORY_MANAGER.with(|m| m.borrow().get(id))
}

// ── Upgrade blob ───────────────────────────────────────────────────────────

/// True when stable memory already uses the MemoryManager layout.
pub fn has_memory_manager_layout() -> bool {
    let raw = DefaultMemoryImpl::default();
    if raw.size() == 0 {
        return false;
    }
    let mut magic = [0u8; 3];
    raw.read(0, &mut magic);
    &magic == b"MGR"
}

/// Writes the heap upgrade blob (length-prefixed) into its virtual memory.
pub fn write_upgrade_blob(bytes: &[u8]) {
    let m = mem(UPGRADE_BLOB_MEM);
    let needed = 8 + bytes.len() as u64;
    let have = m.size() * WASM_PAGE;
    if needed > have {
        let pages = (needed - have).div_ceil(WASM_PAGE);
        if m.grow(pages) < 0 {
            ic_cdk::trap("Failed to grow stable memory for upgrade blob");
        }
    }
    m.write(0, &(bytes.len() as u64).to_le_bytes());
    m.write(8, bytes);
}

pub fn read_upgrade_blob() -> Vec<u8> {
    let m = mem(UPGRADE_BLOB_MEM);
    if m.size() == 0 {
        return vec![];
    }
    let mut len = [0u8; 8];
    m.read(0, &mut len);
    let mut bytes = vec![0u8; u64::from_le_bytes(len) as usize];
    m.read(8, &mut bytes);
    bytes
}

// ── Listings ───────────────────────────────────────────────────────────────

pub fn token_key(collection: &Principal, token_id: &Nat) -> String {
    format!("{}:{}", collection.to_text(), token_id.0.to_str_radix(10))
}

pub fn next_listing_id() -> u64 {
    LISTINGS.with(|l| l.borrow().last_key_value().map_or(0, |(k, _)| k + 1))
}

pub fn get_listing(id: u64) -> Option<Listing> {
    LISTINGS.with(|l| l.borrow().get(&id))
}

/// Saves a listing and keeps the active index in sync with its status.
pub fn save_listing(listing: &Listing) {
    use super::types::ListingStatus::*;
    let key = token_key(&listing.collection, &listing.token_id);
    match listing.status {
        Active | Locked => ACTIVE_LISTINGS.with(|a| {
            a.borrow_mut().insert(key, listing.id);
        }),
        Sold | Cancelled => ACTIVE_LISTINGS.with(|a| {
            let mut a = a.borrow_mut();
            if a.get(&key) == Some(listing.id) {
                a.remove(&key);
            }
        }),
    }
    LISTINGS.with(|l| l.borrow_mut().insert(listing.id, listing.clone()));
}

pub fn active_listing_for(collection: &Principal, token_id: &Nat) -> Option<Listing> {
    let id = ACTIVE_LISTINGS.with(|a| a.borrow().get(&token_key(collection, token_id)))?;
    get_listing(id)
}

/// Active/Locked listings, optionally limited to one collection.
pub fn active_listings(collection: Option<&Principal>) -> Vec<Listing> {
    ACTIVE_LISTINGS.with(|a| {
        let a = a.borrow();
        let ids: Vec<u64> = match collection {
            Some(c) => {
                let prefix = format!("{}:", c.to_text());
                a.range(prefix.clone()..)
                    .take_while(|(k, _)| k.starts_with(&prefix))
                    .map(|(_, v)| v)
                    .collect()
            }
            None => a.iter().map(|(_, v)| v).collect(),
        };
        ids.into_iter().filter_map(get_listing).collect()
    })
}

/// Listings newest first, filtered.
pub fn listings_desc(filter: impl Fn(&Listing) -> bool, offset: u64, limit: u64) -> Vec<Listing> {
    let next = next_listing_id();
    (0..next)
        .rev()
        .filter_map(get_listing)
        .filter(|l| filter(l))
        .skip(offset as usize)
        .take(limit as usize)
        .collect()
}

// ── Transactions ───────────────────────────────────────────────────────────

pub fn next_tx_id() -> u64 {
    TXS.with(|t| t.borrow().last_key_value().map_or(0, |(k, _)| k + 1))
}

pub fn get_tx(id: u64) -> Option<MarketTx> {
    TXS.with(|t| t.borrow().get(&id))
}

/// Saves a tx and keeps the pending index in sync with its status.
pub fn save_tx(tx: &MarketTx) {
    PENDING_TXS.with(|p| {
        let mut p = p.borrow_mut();
        if tx.status.is_terminal() {
            p.remove(&tx.id);
        } else {
            p.insert(tx.id, ());
        }
    });
    TXS.with(|t| t.borrow_mut().insert(tx.id, tx.clone()));
}

pub fn count_txs() -> u64 {
    TXS.with(|t| t.borrow().len())
}

pub fn pending_tx_ids() -> Vec<u64> {
    PENDING_TXS.with(|p| p.borrow().iter().map(|(k, _)| k).collect())
}

/// Txs newest first, filtered.
pub fn txs_desc(filter: impl Fn(&MarketTx) -> bool, offset: u64, limit: u64) -> Vec<MarketTx> {
    let next = next_tx_id();
    (0..next)
        .rev()
        .filter_map(get_tx)
        .filter(|t| filter(t))
        .skip(offset as usize)
        .take(limit as usize)
        .collect()
}

// ── Collections & config ───────────────────────────────────────────────────

pub fn get_collection(id: &Principal) -> Option<NftCollection> {
    COLLECTIONS.with(|c| c.borrow().get(id).cloned())
}

pub fn config() -> Option<MarketplaceConfig> {
    CONFIG.with(|c| c.borrow().clone())
}

// ── Concurrency guards ─────────────────────────────────────────────────────

/// Marks a tx as being processed. Released on drop, including when the
/// message traps (ic-cdk drops the future during cleanup).
pub struct TxGuard(u64);

impl TxGuard {
    pub fn acquire(id: u64) -> Option<Self> {
        IN_FLIGHT.with(|f| f.borrow_mut().insert(id).then_some(Self(id)))
    }
}

impl Drop for TxGuard {
    fn drop(&mut self) {
        IN_FLIGHT.with(|f| f.borrow_mut().remove(&self.0));
    }
}

pub struct ClockGuard;

impl ClockGuard {
    pub fn acquire() -> Option<Self> {
        CLOCK_RUNNING.with(|r| {
            let mut r = r.borrow_mut();
            if *r {
                None
            } else {
                *r = true;
                Some(Self)
            }
        })
    }
}

impl Drop for ClockGuard {
    fn drop(&mut self) {
        CLOCK_RUNNING.with(|r| *r.borrow_mut() = false);
    }
}

#[cfg(test)]
pub fn reset_for_tests() {
    LISTINGS.with(|l| *l.borrow_mut() = StableBTreeMap::new(mem(LISTINGS_MEM)));
    TXS.with(|t| *t.borrow_mut() = StableBTreeMap::new(mem(TXS_MEM)));
    ACTIVE_LISTINGS.with(|a| *a.borrow_mut() = StableBTreeMap::new(mem(ACTIVE_LISTINGS_MEM)));
    PENDING_TXS.with(|p| *p.borrow_mut() = StableBTreeMap::new(mem(PENDING_TXS_MEM)));
    COLLECTIONS.with(|c| c.borrow_mut().clear());
    CONFIG.with(|c| *c.borrow_mut() = None);
    IN_FLIGHT.with(|f| f.borrow_mut().clear());
}
