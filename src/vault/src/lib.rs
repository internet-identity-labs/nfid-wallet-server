extern crate core;
#[macro_use]
extern crate maplit;

use std::convert::TryFrom;

use candid::{candid_method, export_service, Principal};
use ic_cdk::export::candid;
use ic_cdk_macros::*;
use ic_ledger_types::AccountIdentifier;

use crate::enums::TransactionState;
use crate::memory::{Conf, CONF};
use crate::policy_service::{Policy, PolicyType};
use crate::request::{PolicyRegisterRequest, TransactionApproveRequest, TransactionRegisterRequest, VaultMemberRequest, VaultRegisterRequest, WalletRegisterRequest};
use crate::security_service::trap_if_not_permitted;
use crate::transaction_service::Transaction;
use crate::TransactionState::Approved;
use crate::transfer_service::transfer;
use crate::user_service::{get_or_new_by_caller, User};
use crate::util::{caller_to_address, to_array};
use crate::vault_service::{Vault, VaultRole};
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
    user.vaults.insert(vault.id.clone());
    user_service::restore(user);
    vault
}

#[update]
#[candid_method(update)]
async fn update_vault(vault: Vault) -> Vault {
    trap_if_not_permitted(vault.id, vec![VaultRole::Admin]);
    vault_service::update(vault)
}

#[query]
#[candid_method(query)]
async fn get_vaults() -> Vec<Vault> {
    let vault_ids = user_service::get_or_new_by_caller().vaults;
    vault_service::get(vault_ids)
}

#[update]
#[candid_method(update)]
async fn store_member(request: VaultMemberRequest) -> Vault {
    trap_if_not_permitted(request.vault_id, vec![VaultRole::Admin]);
    let mut user = user_service::get_or_new_by_address(request.address);
    let vault = vault_service::add_vault_member(request.vault_id, &user, request.role, request.name, request.state);
    user.vaults.insert(vault.id.clone());
    user_service::restore(user);
    vault_service::restore(vault.clone())
}

#[update]
#[candid_method(update)]
async fn register_wallet(request: WalletRegisterRequest) -> Wallet {
    trap_if_not_permitted(request.vault_id, vec![VaultRole::Admin]);
    let mut vault = vault_service::get_by_id(request.vault_id);
    let new_wallet = wallet_service::new_and_store(request.name, request.vault_id);
    vault.wallets.insert(new_wallet.id);
    vault_service::restore(vault);
    new_wallet
}

#[update]
#[candid_method(update)]
async fn update_wallet(wallet: Wallet) -> Wallet {
    let old = wallet_service::get_by_id(wallet.id);
    for vault_id in &old.vaults {
        trap_if_not_permitted(*vault_id, vec![VaultRole::Admin]);
    }
    wallet_service::update(wallet)
}

#[update]
#[candid_method(update)]
async fn register_policy(request: PolicyRegisterRequest) -> Policy {
    trap_if_not_permitted(request.vault_id, vec![VaultRole::Admin]); //todo accepted_wallets?
    let mut vault = vault_service::get_by_id(request.vault_id);
    let policy = policy_service::register_policy(request.vault_id, request.policy_type);
    vault.policies.insert(policy.id);
    vault_service::restore(vault);
    policy
}

#[query]
#[candid_method(query)]
async fn get_wallets(vault_id: u64) -> Vec<Wallet> {
    trap_if_not_permitted(vault_id, vec![VaultRole::Admin, VaultRole::Member]);
    let vault = vault_service::get(hashset![vault_id]);
    wallet_service::get_wallets(vault[0].wallets.clone())
}


#[query]
#[candid_method(query)]
async fn get_policies(vault_id: u64) -> Vec<Policy> {
    trap_if_not_permitted(vault_id, vec![VaultRole::Admin, VaultRole::Member]);
    let vault = vault_service::get(hashset![vault_id]);
    policy_service::get(vault[0].policies.clone())
}

#[update]
#[candid_method(update)]
async fn update_policy(policy: Policy) -> Policy {
    let old = policy_service::get_by_id(policy.id);
    trap_if_not_permitted(old.vault, vec![VaultRole::Admin]); //todo new wallets relation to vault?
    policy_service::update_policy(policy)
}

#[update]
#[candid_method(update)]
async fn register_transaction(request: TransactionRegisterRequest) -> Transaction {
    let wallet = wallet_service::get_by_id(request.wallet_id);
    let vaults = vault_service::get(wallet.vaults.clone());
    let vault = vaults.first().unwrap(); //for now one2one
    trap_if_not_permitted(vault.id, vec![VaultRole::Admin, VaultRole::Member]);
    let policy = policy_service::define_correct_policy(vault.policies.clone(), request.amount, request.wallet_id);
    let transaction = transaction_service::register_transaction(request.amount, request.address, request.wallet_id, policy);
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
    let mut claimed_transaction = transaction_service::claim_transaction(ts, request.state);
    if Approved.eq(&claimed_transaction.state) {
        let subaccount = wallet_service::id_to_subaccount(claimed_transaction.wallet_id);
        let result = transfer(claimed_transaction.amount, claimed_transaction.to.clone(), subaccount).await;
        match result {
            Ok(block) => {
                claimed_transaction.block_index = Some(block);
                transaction_service::store_transaction(claimed_transaction.clone());
            }
            Err(_) => {
                claimed_transaction.state = TransactionState::Rejected;
                transaction_service::store_transaction(claimed_transaction.clone());
                // trap(e.as_str()) //TODO: add reason?
            }
        }
    }
    claimed_transaction
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
