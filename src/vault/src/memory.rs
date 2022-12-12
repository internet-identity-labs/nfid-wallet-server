use std::collections::HashMap;
use ic_cdk::storage;
use crate::{POLICIES, Policy, Transaction, TRANSACTIONS, User, USERS, Vault, VAULTS, Wallet, WALLETS};
use ic_cdk::export::{candid::{CandidType, Deserialize}};

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct VaultMemoryObject {
    pub vaults: Vec<Vault>,
    pub users: Vec<User>,
    pub wallets: Vec<Wallet>,
    pub transactions: Vec<Transaction>,
    pub policies: Vec<Policy>,
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
