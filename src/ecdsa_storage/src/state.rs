use std::borrow::Cow;
use std::cell::RefCell;

use crate::legacy_signer::{Salts, StoredKeyPair};
use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell, Storable};

type Memory = VirtualMemory<DefaultMemoryImpl>;

const CONFIG_MEMORY: MemoryId = MemoryId::new(0);
const GLOBAL_KEYS_MEMORY: MemoryId = MemoryId::new(1);

#[derive(CandidType, Deserialize, Clone, Default)]
pub struct Config {
    pub im_canister: Option<Principal>,
    /// DER-encoded IC root key; the mainnet key when absent.
    pub ic_root_key: Option<Vec<u8>>,
    /// X25519 secret used once to receive the salts.
    pub provisioning_secret: Option<Vec<u8>>,
    /// Principal allowed to load the salts and import the global keys once, without being a
    /// controller: the migration lambda. Revoke it when the migration is done.
    pub migrator: Option<Principal>,
    pub ecdsa_salt: Option<String>,
    pub anonymous_salt: Option<String>,
}

/// A global key record copied from `signer_ic`.
#[derive(CandidType, Deserialize, Clone)]
pub struct GlobalKey {
    pub public_key: String,
    pub private_key_encrypted: String,
}

macro_rules! candid_storable {
    ($type:ty) => {
        impl Storable for $type {
            fn to_bytes(&self) -> Cow<'_, [u8]> {
                Cow::Owned(Encode!(self).expect("candid encoding cannot fail"))
            }

            fn from_bytes(bytes: Cow<[u8]>) -> Self {
                Decode!(&bytes, Self).expect("stored value must be valid candid")
            }

            const BOUND: Bound = Bound::Unbounded;
        }
    };
}

candid_storable!(Config);
candid_storable!(GlobalKey);

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static CONFIG: RefCell<StableCell<Config, Memory>> = RefCell::new(
        StableCell::init(memory(CONFIG_MEMORY), Config::default())
            .expect("cannot initialize the config cell"),
    );

    /// Root principal -> encrypted global key.
    static GLOBAL_KEYS: RefCell<StableBTreeMap<String, GlobalKey, Memory>> =
        RefCell::new(StableBTreeMap::init(memory(GLOBAL_KEYS_MEMORY)));
}

fn memory(id: MemoryId) -> Memory {
    MEMORY_MANAGER.with(|manager| manager.borrow().get(id))
}

pub fn config() -> Config {
    CONFIG.with(|cell| cell.borrow().get().clone())
}

pub fn update_config(update: impl FnOnce(&mut Config)) {
    CONFIG.with(|cell| {
        let mut cell = cell.borrow_mut();
        let mut config = cell.get().clone();
        update(&mut config);
        cell.set(config).expect("cannot store the config");
    });
}

pub fn salts() -> Option<Salts> {
    let config = config();
    Some(Salts {
        ecdsa_salt: config.ecdsa_salt?,
        anonymous_salt: config.anonymous_salt?,
    })
}

pub fn global_key(root_principal: &str) -> Option<StoredKeyPair> {
    GLOBAL_KEYS.with(|keys| {
        keys.borrow()
            .get(&root_principal.to_string())
            .map(|key| StoredKeyPair {
                public_key: key.public_key,
                private_key_encrypted: key.private_key_encrypted,
            })
    })
}

pub fn insert_global_key(root_principal: String, key: GlobalKey) {
    GLOBAL_KEYS.with(|keys| keys.borrow_mut().insert(root_principal, key));
}

pub fn global_keys_count() -> u64 {
    GLOBAL_KEYS.with(|keys| keys.borrow().len())
}
