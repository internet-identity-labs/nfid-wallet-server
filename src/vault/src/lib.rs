extern crate core;

use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::convert::{TryFrom, TryInto};
use std::fmt::Debug;
use std::hash::Hash;

use candid::{candid_method, export_service, Principal};
use ic_cdk::{storage, trap};
use ic_cdk::export::{candid::{CandidType, Deserialize}};
use ic_cdk::export::candid;
use ic_cdk_macros::*;
use ic_ledger_types::{AccountIdentifier, MAINNET_LEDGER_CANISTER_ID, Subaccount};

use crate::enums::State;
use crate::policy_service::{Policy, PolicyType};
use crate::security_service::trap_if_not_permitted;
use crate::transaction_service::Transaction;
use crate::transfer_service::transfer;
use crate::user_service::{get_or_new_by_caller, User};
use crate::util::caller_to_address;
use crate::vault_service::{Vault, VaultMember, VaultRole};
use crate::wallet_service::{id_to_address, id_to_subaccount, Wallet};

mod user_service;
mod vault_service;
mod wallet_service;
mod policy_service;
mod transaction_service;
mod notification_service;
mod util;
mod enums;
mod security_service;
mod transfer_service;

#[derive(CandidType, Deserialize, Clone, Debug, Hash, PartialEq)]
pub struct Conf {
    ledger_canister_id: Principal,
}


impl Default for Conf {
    fn default() -> Self {
        Conf {
            ledger_canister_id: MAINNET_LEDGER_CANISTER_ID,
        }
    }
}


thread_local! {
    static CONF: RefCell<Conf> = RefCell::new(Conf::default());
    static USERS: RefCell<HashMap<String, User >> = RefCell::new(Default::default());
    static VAULTS: RefCell< HashMap<u64, Vault>> = RefCell::new(Default::default());
    static WALLETS: RefCell< HashMap<u64, Wallet>> = RefCell::new(Default::default());
    static POLICIES: RefCell< HashMap<u64, Policy>> = RefCell::new(Default::default());
    static TRANSACTIONS: RefCell< HashMap<u64, Transaction>> = RefCell::new(Default::default());
}
#[init]
#[candid_method(init)]
fn init(conf: Option<Conf>) {
    match conf {
        None => {}
        Some(conf) => {
            CONF.with(|c| c.replace(conf));
        }
    };
}

#[query]
#[candid_method(query, rename = "sub")]
async fn sub(wallet_id: u64) -> String {
    id_to_address(wallet_id).to_string()
}

#[query]
#[candid_method(query, rename = "sub_vec")]
async fn sub_vec(wallet_id: u64) -> Subaccount {
    id_to_subaccount(wallet_id)
}

#[query]
#[candid_method(query, rename = "sub_bytes")]
async fn sub_bytes(wallet_id: u64) -> AccountIdentifier {
    id_to_address(wallet_id)
}


#[derive(CandidType, Deserialize, Clone)]
pub struct VaultRegisterRequest {
    name: String,
    description: Option<String>,
}

#[update]
#[candid_method(update)]
async fn register_vault(request: VaultRegisterRequest) -> Vault {
    let address = caller_to_address();
    let mut user = user_service::get_or_new_by_address(address);
    let vault = vault_service::register(user.address.clone(), request.name, request.description);
    user.vaults.push(vault.id.clone());
    user_service::restore(user);
    vault
}

#[query]
#[candid_method(query)]
async fn get_vaults() -> Vec<Vault> {
    let vault_ids = user_service::get_or_new_by_caller().vaults;
    vault_service::get(vault_ids)
}


#[derive(CandidType, Deserialize, Clone)]
pub struct VaultMemberRequest {
    vault_id: u64,
    address: String,
    name: Option<String>,
    role: VaultRole,
}

#[update]
#[candid_method(update)]
async fn add_vault_member(request: VaultMemberRequest) -> Vault {
    trap_if_not_permitted(request.vault_id, vec![VaultRole::Admin]);
    let mut vault = vault_service::get_by_id(request.vault_id);
    let mut user = user_service::get_or_new_by_address(request.address);
    let vm = VaultMember {
        user_uuid: user.address.clone(),
        role: request.role,
        name: request.name,
    };
    vault.members.insert(vm);
    user.vaults.push(vault.id.clone());
    user_service::restore(user);
    vault_service::restore(vault.clone());
    vault
}

#[query]
#[candid_method(query)]
async fn get_vault_members(vault_id: u64) -> HashSet<VaultMember> {
    trap_if_not_permitted(vault_id, vec![VaultRole::Admin, VaultRole::Member]);
    vault_service::get_by_id(vault_id).members //todo??
}

#[derive(CandidType, Deserialize, Clone)]
pub struct WalletRegisterRequest {
    vault_id: u64,
    name: Option<String>,
}

#[update]
#[candid_method(update)]
async fn register_wallet(request: WalletRegisterRequest) -> Wallet {
    trap_if_not_permitted(request.vault_id, vec![VaultRole::Admin]);
    let mut vault = vault_service::get_by_id(request.vault_id);
    let new_wallet = wallet_service::new_and_store(request.name, request.vault_id);
    vault.wallets.push(new_wallet.id);
    vault_service::restore(vault);
    new_wallet
}


#[derive(CandidType, Deserialize, Clone)]
pub struct PolicyRegisterRequest {
    vault_id: u64,
    policy_type: PolicyType,
}


#[update]
#[candid_method(update)]
async fn register_policy(request: PolicyRegisterRequest) -> Policy {
    trap_if_not_permitted(request.vault_id, vec![VaultRole::Admin]);
    let mut vault = vault_service::get_by_id(request.vault_id);
    let policy = policy_service::register_policy(request.policy_type);
    vault.policies.push(policy.id);
    vault_service::restore(vault);
    policy
}

#[query]
#[candid_method(query)]
async fn get_wallets(vault_id: u64) -> Vec<Wallet> {
    trap_if_not_permitted(vault_id, vec![VaultRole::Admin, VaultRole::Member]);
    let vault = vault_service::get(vec![vault_id]);
    wallet_service::get_wallets(vault[0].wallets.clone())
}


#[query]
#[candid_method(query)]
async fn get_policies(vault_id: u64) -> Vec<Policy> {
    trap_if_not_permitted(vault_id, vec![VaultRole::Admin, VaultRole::Member]);
    let vault = vault_service::get(vec![vault_id]);
    policy_service::get(vault[0].policies.clone())
}


#[derive(CandidType, Deserialize, Clone)]
pub struct TransactionRegisterRequest {
    amount: u64,
    address: String,
    wallet_id: u64,
}


#[update]
#[candid_method(update)]
async fn register_transaction(request: TransactionRegisterRequest) -> Transaction {
    let wallet = wallet_service::get_wallet(request.wallet_id);
    let vaults = vault_service::get(wallet.vaults.clone());
    let vault = vaults.first().unwrap(); //one to one??
    trap_if_not_permitted(vault.id, vec![VaultRole::Admin, VaultRole::Member]);
    let policy = policy_service::get(vault.policies.clone());
    let transaction = transaction_service::register_transaction(request.amount, request.address, request.wallet_id, policy.first().unwrap().clone(), vault.id);
    transaction
}

#[query]
#[candid_method(query)]
async fn get_transactions() -> Vec<Transaction> {
    let tr_owner = user_service::get_or_new_by_caller();
    return transaction_service::get_all(tr_owner.vaults);
}


#[derive(CandidType, Deserialize, Clone)]
pub struct TransactionApproveRequest {
    transaction_id: u64,
    state: State,
}

#[update]
#[candid_method(update)]
async fn approve_transaction(request: TransactionApproveRequest) -> Transaction {//TODO: synonym to approve ??? claim_transaction/affect_transaction
    let tr_owner = user_service::get_or_new_by_caller();
    let mut transaction = transaction_service::approve_transaction(request.transaction_id, tr_owner, request.state);
    if transaction.approves.clone()
        .into_iter()
        .filter(|l| l.status.eq(&State::APPROVED))
        .count() as u8 >= transaction.member_threshold {
        let subaccount = wallet_service::id_to_subaccount(transaction.wallet_id);
        let decoded = hex::decode(transaction.to.clone()).unwrap();
        let to: AccountIdentifier = AccountIdentifier::try_from(to_array(decoded)).unwrap();
        let result = transfer(transaction.amount, to, subaccount).await;
        match result {
            Ok(block) => {
                transaction.block_index = Some(block);
                transaction.state = State::APPROVED;
                transaction_service::store_transaction(transaction.clone());
            }
            Err(e) => {
                transaction.state = State::REJECTED;
                transaction_service::store_transaction(transaction.clone());
                trap(e.as_str()) //TODO: add reason?
            }
        }
    }
    transaction
}

fn to_array<T>(v: Vec<T>) -> [T; 32] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", 32, v.len()))
}


#[test]
fn sub_account_test() {
    let account_ad = 18_446_744_073_709_551_615u64 as u64;
    let a = id_to_subaccount(account_ad);
    let tt = AccountIdentifier::new(&Principal::from_text("ymvb6-7qaaa-aaaan-qbgga-cai".to_string()).unwrap(), &a);
    print!("{} \n", tt.to_string());
    let yyy = to_array(hex::decode(tt.to_string()).unwrap());
    let tty: AccountIdentifier = AccountIdentifier::try_from(yyy).unwrap();
    print!("{} ", tty.to_string());
    assert_eq!(tty, tt)
}

export_service!();

#[ic_cdk_macros::query(name = "__get_candid_interface")]
fn export_candid() -> String {
    __export_service()
}


#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct VaultMemoryObject {
    pub vaults: Vec<Vault>,
    pub users: Vec<User>,
    pub wallets: Vec<Wallet>,
    pub transactions: Vec<Transaction>,
    pub policies: Vec<Policy>,
}

#[pre_upgrade]
fn pre_upgrade() {
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


#[post_upgrade]
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
