mod address_book;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use candid::{CandidType, Principal};
use candid::export_service;
use ic_cdk::{call, caller, storage, trap};
use ic_cdk_macros::*;
use serde::{Deserialize, Serialize};

pub use address_book::*;

#[derive(CandidType, Deserialize, Clone, Serialize, Debug, Hash, PartialEq)]
pub struct Conf {
    pub im_canister: Option<String>,
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug, Hash, PartialEq, Eq)]
pub enum ICRC1State {
    Active,
    Inactive,
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug, Eq)]
pub struct ICRC1 {
    pub state: ICRC1State,
    pub ledger: String,
    pub network: u32,
}

impl Hash for ICRC1 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ledger.hash(state);
        self.network.hash(state);
    }
}

impl PartialEq for ICRC1 {
    fn eq(&self, other: &Self) -> bool {
        self.ledger == other.ledger && self.network == other.network
    }
}

const DEFAULT_ADDRESS_BOOK_CONFIG: AddressBookConf = AddressBookConf {
    max_user_addresses: 50,
    max_name_length: 200,
};

thread_local! {
     static CONFIG: RefCell<Conf> = const { RefCell::new( Conf {
        im_canister: None
    }) };
    pub static ICRC_REGISTRY: RefCell<HashMap<String, HashSet<ICRC1>>> = RefCell::new(HashMap::default());

    pub(crate) static ADDRESS_BOOK: RefCell<HashMap<String, AddressBookUser>> = RefCell::new(HashMap::default());
    pub(crate) static ADDRESS_BOOK_CONFIG: RefCell<AddressBookConf> = const { RefCell::new(DEFAULT_ADDRESS_BOOK_CONFIG) };
}


/// Persists the ICRC1 canister metadata for a specified user ledger ID principal.
#[update]
pub async fn store_icrc1_canister(ledger_id: String, state: ICRC1State, network: Option<u32>) {
    let caller = get_root_id().await;
    ICRC_REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        let canister = ICRC1 {
            state,
            ledger: ledger_id.clone(),
            network: network.unwrap_or(0),
        };
        let canisters = registry.entry(caller).or_insert_with(HashSet::new);
        let network_value = network.unwrap_or(0);
        canisters.retain(|existing_canister| !(existing_canister.ledger == ledger_id && existing_canister.network == network_value));
        canisters.insert(canister);
    });
}

#[update]
pub async fn remove_icrc1_canister(ledger_id: String, network: Option<u32>) {
    let caller = get_root_id().await;
    ICRC_REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        if let Some(canisters) = registry.get_mut(&caller) {
            canisters.retain(|existing_canister| !(existing_canister.ledger == ledger_id && existing_canister.network == network.unwrap_or(0)));
        }
    });
}

/// Invoked when the canister starts.
/// Initializes the application with `Conf` parameters and saves them to storage.
#[init]
pub async fn init(conf: Conf) {
    CONFIG.with(|c| c.replace(conf));
}

/// Returns all ICRC1 canisters persisted for the specified ledger ID principal.
#[query]
pub async fn get_canisters_by_root(root: String) -> Vec<ICRC1> {
    ICRC_REGISTRY.with(|registry| {
        let registry = registry.borrow();
        registry.get(&root).cloned().unwrap_or_default()
            .into_iter().collect()
    })
}

#[update]
pub async fn address_book_save(user_address: AddressBookUserAddress) -> Result<Vec<AddressBookUserAddress>, AddressBookError> {
    let root_id = get_root_id().await;
    address_book::service::save(root_id, user_address).await
}

#[update]
pub async fn address_book_delete(id: String) -> Result<Vec<AddressBookUserAddress>, AddressBookError> {
    let root_id = get_root_id().await;
    address_book::service::delete(root_id, id).await
}

#[update]
pub async fn address_book_delete_all() -> Result<(), AddressBookError> {
    let root_id = get_root_id().await;
    address_book::service::delete_all(root_id).await
}

#[query(composite = true)]
pub async fn address_book_find_all() -> Result<Vec<AddressBookUserAddress>, AddressBookError> {
    let root_id = get_root_id().await;
    address_book::service::find_all(root_id).await
}

#[query]
pub fn address_book_get_config() -> AddressBookConf {
    address_book::service::get_config()
}

#[update]
pub async fn address_book_set_config(config: AddressBookConf) -> Result<(), AddressBookError> {
    address_book::service::set_config(config).await
}


#[derive(CandidType, Deserialize, Clone, Serialize, Debug, PartialEq, Eq, Hash)]
pub struct ICRC1Memory {
    pub state: ICRC1State,
    pub ledger: String,
    pub network: Option<u32>,
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
struct Memory {
    registry: HashMap<String, HashSet<ICRC1Memory>>,
    config: Conf,
    address_book: HashMap<String, AddressBookUser>,
    address_book_config: Option<AddressBookConf>,
}

/// Applies changes before the canister upgrade.
#[pre_upgrade]
pub fn stable_save() {
    let registry: HashMap<String, HashSet<ICRC1>> = ICRC_REGISTRY.with(|registry| {
        let registry = registry.borrow();
        registry.clone()
    });
    let config = CONFIG.with(|config| {
        let config = config.borrow();
        config.clone()
    });
    let address_book = ADDRESS_BOOK.with(|book| book.borrow().clone());
    let address_book_config = ADDRESS_BOOK_CONFIG.with(|c| c.borrow().clone());

    let registry: HashMap<String, HashSet<ICRC1Memory>> = registry.into_iter().map(|(k, v)| (k, v.into_iter().map(|x| ICRC1Memory {
        state: x.state,
        ledger: x.ledger.clone(),
        network: Some(x.network),
    }).collect())).collect();
    let mem = Memory {
        registry,
        config,
        address_book,
        address_book_config: Some(address_book_config),
    };
    storage::stable_save((mem,)).expect("Stable save exited unexpectedly: unable to save data to stable memory.");
}

/// Applies changes after the canister upgrade.
#[post_upgrade]
pub fn stable_restore() {
    let (mem, ): (Memory, ) = storage::stable_restore()
        .expect("Stable restore exited unexpectedly: unable to restore data from stable memory.");

    let Memory { config, registry, address_book, address_book_config } = mem;

    CONFIG.with(|c| {
        *c.borrow_mut() = config.clone();
    });
    ICRC_REGISTRY.with(|reg| {
        let mut reg = reg.borrow_mut();
        *reg = registry.into_iter().map(|(k, v)| (k, v.into_iter().map(|x| ICRC1 {
            state: x.state,
            ledger: x.ledger,
            network: x.network.unwrap_or(0),
        }).collect())).collect();
    });
    ADDRESS_BOOK.with(|book| {
        *book.borrow_mut() = address_book;
    });
    ADDRESS_BOOK_CONFIG.with(|c| {
        *c.borrow_mut() = address_book_config.unwrap_or(DEFAULT_ADDRESS_BOOK_CONFIG);
    });
}


#[test]
fn sub_account_test() {}
export_service!();

#[ic_cdk_macros::query(name = "__get_candid_interface")]
fn export_candid() -> String {
    __export_service()
}


async fn get_root_id() -> String {
    match CONFIG.with(|c| c.borrow_mut().im_canister.clone()) {
        None => caller().to_text(), // Return caller for testing purposes when im_canister is None
        Some(canister) => {
            let princ = caller();
            let im_canister = Principal::from_text(canister)
                .expect("Unable to obtain Principal from im_canister.");

            match call(im_canister, "get_root_by_principal", (princ.to_text(), 0)).await {
                Ok((Some(root_id), )) => root_id,
                Ok((None, )) => trap("No root found for this principal"),
                Err((_, err)) => trap(&format!("Failed to request IM: {}", err)),
            }
        }
    }
}
