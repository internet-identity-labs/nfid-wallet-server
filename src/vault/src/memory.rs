use candid::Principal;
use ic_cdk::storage;
use crate::{Policy, Transaction, User, Vault, Wallet};
use ic_cdk::export::{candid::{CandidType, Deserialize}};
use ic_ledger_types::MAINNET_LEDGER_CANISTER_ID;
use std::cell::RefCell;
use std::collections::{HashMap};

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct VaultMemoryObject {
    pub vaults: Vec<Vault>,
    pub users: Vec<User>,
    pub wallets: Vec<Wallet>,
    pub transactions: Vec<Transaction>,
    pub policies: Vec<Policy>,
}


#[derive(CandidType, Deserialize, Clone, Debug, Hash, PartialEq)]
pub struct Conf {
   pub ledger_canister_id: Principal,
}


impl Default for Conf {
    fn default() -> Self {
        Conf {
            ledger_canister_id: MAINNET_LEDGER_CANISTER_ID,
        }
    }
}

//todo make private
thread_local! {
    pub static CONF: RefCell<Conf> = RefCell::new(Conf::default());
    pub static USERS: RefCell<HashMap<String, User>> = RefCell::new(Default::default());
    pub static VAULTS: RefCell<HashMap<u64, Vault>> = RefCell::new(Default::default());
    pub static WALLETS: RefCell<HashMap<u64, Wallet>> = RefCell::new(Default::default());
    pub static POLICIES: RefCell<HashMap<u64, Policy>> = RefCell::new(Default::default());
    pub static TRANSACTIONS: RefCell<HashMap<u64, Transaction>> = RefCell::new(Default::default());
}

pub fn pre_upgrade() {
    let vaults: Vec<Vault> = VAULTS.with(|vaults| {
        vaults.borrow()
            .iter()
            .map(|l| l.1.clone())
            .collect()
    });
    let wallets: Vec<Wallet> = WALLETS.with(|wallets| {
        wallets.borrow()
            .iter()
            .map(|l| l.1.clone())
            .collect()
    });
    let users: Vec<User> = USERS.with(|users| {
        users.borrow()
            .iter()
            .map(|l| l.1.clone())
            .collect()
    });
    let transactions: Vec<Transaction> = TRANSACTIONS.with(|transactions| {
        transactions.borrow()
            .iter()
            .map(|l| l.1.clone())
            .collect()
    });
    let policies: Vec<Policy> = POLICIES.with(|policies| {
        policies.borrow()
            .iter()
            .map(|l| l.1.clone())
            .collect()
    });
    let memory = VaultMemoryObject {
        vaults,
        users,
        wallets,
        policies,
        transactions,
    };
    storage::stable_save((memory, )).unwrap();
}


pub fn post_upgrade() {
    let (mo, ): (VaultMemoryObject, ) = storage::stable_restore().unwrap();
    let mut vaults: HashMap<u64, Vault> = Default::default();
    for vault in mo.vaults {
        vaults.insert(vault.id, vault);
    }
    let mut wallets: HashMap<u64, Wallet> = Default::default();
    for wallet in mo.wallets {
        wallets.insert(wallet.id, wallet);
    }
    let mut users: HashMap<String, User> = Default::default();
    for user in mo.users {
        users.insert(user.address.clone(), user);
    }
    let mut policies: HashMap<u64, Policy> = Default::default();
    for policy in mo.policies {
        policies.insert(policy.id, policy);
    }
    let mut transactions: HashMap<u64, Transaction> = Default::default();
    for transaction in mo.transactions {
        transactions.insert(transaction.id, transaction);
    }
    VAULTS.with(|storage| *storage.borrow_mut() = vaults);
    USERS.with(|storage| *storage.borrow_mut() = users);
    WALLETS.with(|storage| *storage.borrow_mut() = wallets);
    POLICIES.with(|storage| *storage.borrow_mut() = policies);
    TRANSACTIONS.with(|storage| *storage.borrow_mut() = transactions);
}
