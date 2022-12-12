extern crate core;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::convert::{TryFrom};
use std::fmt::Debug;
use std::hash::Hash;

use candid::{candid_method, export_service, Principal};
use ic_cdk::export::{candid::{CandidType, Deserialize}};
use ic_cdk::export::candid;
use ic_cdk_macros::*;
use ic_ledger_types::{AccountIdentifier, MAINNET_LEDGER_CANISTER_ID, Subaccount};

use crate::enums::State;
use crate::memory::{Conf, CONF};
use crate::policy_service::{Policy, PolicyType};
use crate::request::{PolicyRegisterRequest, TransactionApproveRequest, TransactionRegisterRequest, VaultMemberRequest, VaultRegisterRequest, WalletRegisterRequest};
use crate::security_service::trap_if_not_permitted;
use crate::transaction_service::{is_transaction_approved, Transaction};
use crate::transfer_service::transfer;
use crate::user_service::{get_or_new_by_caller, User};
use crate::util::{caller_to_address, to_array};
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
mod request;
mod memory;

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
    vault_service::restore(vault.clone())
}

#[query]
#[candid_method(query)]
async fn get_vault_members(vault_id: u64) -> HashSet<VaultMember> {
    trap_if_not_permitted(vault_id, vec![VaultRole::Admin, VaultRole::Member]);
    vault_service::get_by_id(vault_id).members //todo??
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


#[update]
#[candid_method(update)]
async fn register_policy(request: PolicyRegisterRequest) -> Policy {
    trap_if_not_permitted(request.vault_id, vec![VaultRole::Admin]); //todo accepted_wallets
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


#[update]
#[candid_method(update)]
async fn register_transaction(request: TransactionRegisterRequest) -> Transaction {
    let wallet = wallet_service::get_wallet(request.wallet_id);
    let vaults = vault_service::get(wallet.vaults.clone());
    let vault = vaults.first().unwrap(); //for now one2one
    trap_if_not_permitted(vault.id, vec![VaultRole::Admin, VaultRole::Member]);
    let policy = policy_service::define_correct_policy(vault.policies.clone(), request.amount, request.wallet_id);
    let transaction = transaction_service::register_transaction(request.amount, request.address, request.wallet_id, policy, vault.id);
    transaction
}

#[query]
#[candid_method(query)]
async fn get_transactions() -> Vec<Transaction> {
    let tr_owner = user_service::get_or_new_by_caller();
    return transaction_service::get_all(tr_owner.vaults);
}


#[update]
#[candid_method(update)]
async fn approve_transaction(request: TransactionApproveRequest) -> Transaction {//TODO: synonym to approve ??? claim_transaction/affect_transaction

    let ts = transaction_service::get_by_id(request.transaction_id);
    trap_if_not_permitted(ts.vault_id, vec![VaultRole::Admin, VaultRole::Member]);

    let mut approved_transaction = transaction_service::approve_transaction(ts, request.state);
    if is_transaction_approved(&approved_transaction) {
        let subaccount = wallet_service::id_to_subaccount(approved_transaction.wallet_id);
        let decoded = hex::decode(approved_transaction.to.clone()).unwrap();
        let to: AccountIdentifier = AccountIdentifier::try_from(to_array(decoded)).unwrap();
        let result = transfer(approved_transaction.amount, to, subaccount).await;
        match result {
            Ok(block) => {
                approved_transaction.block_index = Some(block);
                approved_transaction.state = State::APPROVED;
                transaction_service::store_transaction(approved_transaction.clone());
            }
            Err(_) => {
                approved_transaction.state = State::REJECTED;
                transaction_service::store_transaction(approved_transaction.clone());
                // trap(e.as_str()) //TODO: add reason?
            }
        }
    }
    approved_transaction
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


#[pre_upgrade]
fn pre_upgrade() {
    memory::pre_upgrade()
}


#[post_upgrade]
pub fn post_upgrade() {
    memory::post_upgrade()
}
