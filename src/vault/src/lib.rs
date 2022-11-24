extern crate core;

use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use candid::{candid_method, export_service, Principal};
use ic_cdk::{caller, trap};
use ic_cdk::export::candid::CandidType;
use ic_cdk_macros::*;
use ic_ledger_types::{AccountIdentifier, BlockIndex, DEFAULT_FEE, MAINNET_LEDGER_CANISTER_ID, Memo, Subaccount, Tokens};
use serde::{Deserialize, Serialize};

use crate::policy_service::{is_passed, Policy};
use crate::transaction_service::Transaction;
use crate::user_service::User;
use crate::vault_service::{Vault, VaultMember, VaultRole};
use crate::wallet_service::{id_to_subaccount, Wallet, Wallets};

mod user_service;
mod vault_service;
mod wallet_service;
mod policy_service;
mod transaction_service;
mod notification_service;


#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Hash)]
pub struct TransferArgs {
    amount: Tokens,
    to_principal: Principal,
    to_subaccount: Option<Subaccount>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Hash, PartialEq)]
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
    static USERS: RefCell<HashMap<String, User>> = RefCell::new(Default::default());
    static VAULTS: RefCell< HashMap<u64, Vault>> = RefCell::new(Default::default());
    static WALLETS: RefCell< HashMap<u64, Wallet>> = RefCell::new(Default::default());
    static POLICIES: RefCell< HashMap<u64, Policy>> = RefCell::new(Default::default());
    static TRANSACTIONS: RefCell< HashMap<u64, Transaction>> = RefCell::new(Default::default());
}
#[init]
#[candid_method(init)]
fn init(conf: Conf) {
    CONF.with(|c| c.replace(conf));
}

#[query]
#[candid_method(query, rename = "sub")]
async fn sub(princ: String, group: u64, acc: u8) -> String {
    let sixty_fours: [u64; 4] = [group; 4];
    let mut eights: [u8; 32] = bytemuck::cast(sixty_fours);
    eights[31] = eights[31] + acc;
    let to_subaccount = Subaccount(eights);
    AccountIdentifier::new(Principal::from_text(princ).unwrap().borrow(), &to_subaccount).to_string()
}

#[update]
#[candid_method(update)]
async fn register_vault(name: String) -> Vault {
    vault_service::register(name)
}

#[query]
#[candid_method(query)]
async fn get_vaults() -> Vec<Vault> {
    let vault_ids = user_service::get_by_address(caller().to_text()).vaults;
    vault_service::get_by_ids(vault_ids)
}


#[update]
#[candid_method(update)]
async fn register_participant(group_id: u64, address: String, role: VaultRole) -> Vault {
    VAULTS.with(|vaults| {
        return match vaults.borrow_mut().get_mut(&group_id) {
            None => { trap("Group not exists") }
            Some(vault) => {
                let user = user_service::get_or_new_by_address(address, group_id);
                let vm = VaultMember { user_id: user.id, role };
                vault.participants.push(vm);
                vault.clone()
            }
        };
    })
}

#[update]
#[candid_method(update)]
async fn register_wallet(vault_id: u64, wallet_name: Option<String>) -> Wallet {
    let user = user_service::get_or_new_by_address(caller().to_text(), vault_id);
    //todo check
    let mut vault = match vault_service::get_by_ids(vec![vault_id]).first() {
        None => { trap("   ") }
        Some(vault) => {
            match vault.participants
                .iter()
                .find(|p| user.id.eq(&p.user_id)) {
                None => {
                    trap("Unauthorised")
                }
                Some(vaultMember) => {
                    match vaultMember.role {
                        VaultRole::VaultOwner => {}
                        VaultRole::VaultApprove => {
                            trap("Not enough permissions")
                        }
                    }
                }
            }
            vault.clone()
        }
    };

    let new_wallet = wallet_service::new_and_store(wallet_name, vault_id);
    vault.wallets.push(new_wallet.id);
    vault_service::restore(vault);
    new_wallet
}


#[update]
async fn register_transaction(amount: Tokens, to: AccountIdentifier, wallet_id: u64) -> Transaction {
    let caller = caller().to_text();
    let tr_owner = user_service::get_by_address(caller);
    let mut wallet = wallet_service::get_wallet(wallet_id);

    let vault = vault_service::get_by_ids(wallet.vault_ids.clone());

    let user_id = tr_owner.id.clone();

    let policy = policy_service::get_by_ids(vault[0].policies.clone());


    let transaction = transaction_service::register_transaction(amount, to, wallet_id, tr_owner, policy.first().unwrap().clone());

    wallet.transaction_ids.push(transaction.id);//todo think about index
    wallet_service::restore(wallet);

    let users_to_notify = vault.into_iter()
        .map(|l| l.participants)
        .flat_map(|k| k)
        .map(|k| k.user_id)
        .filter(|l| !l.eq(&user_id))
        .collect();

    notification_service::register_notification(transaction.id, users_to_notify);
    transaction
}

//
// #[update]
// async fn get_transactions() -> Transaction {
//     let caller = caller().to_text();
//     let tr_owner = user_service::get_by_address(caller);
//
// }
//
//
// #[update]
// async fn get_notifications() -> Transaction {
//     let caller = caller().to_text();
//     let tr_owner = user_service::get_by_address(caller);
//
// }


#[update]
async fn approve_transaction(transaction_id: u64) -> Transaction {
    let caller = caller().to_text();
    let tr_owner = user_service::get_by_address(caller);
    let mut transaction = transaction_service::approve_transaction(transaction_id, tr_owner);
    if is_passed(transaction.clone()) {
        let subaccount = wallet_service::id_to_subaccount(transaction.wallet_id);
        let result = transfer(transaction.amount, transaction.to, subaccount).await;
        match result {
            Ok(block) => {
                transaction.block_index = Some(block);
                transaction_service::store_transaction(transaction.clone());
            }
            Err(e) => {
                trap(e.as_str())
            }
        }
    }
    transaction
}


async fn transfer(amount: Tokens, to: AccountIdentifier, from_subaccount: Subaccount) -> Result<BlockIndex, String> {
    let ledger_canister_id = MAINNET_LEDGER_CANISTER_ID;
    let transfer_args = ic_ledger_types::TransferArgs {
        memo: Memo(0),
        amount,
        fee: DEFAULT_FEE,
        from_subaccount: Some(from_subaccount),
        to,
        created_at_time: None,
    };
    ic_ledger_types::transfer(ledger_canister_id, transfer_args).await
        .map_err(|e| format!("failed to call ledger: {:?}", e))?
        .map_err(|e| format!("ledger transfer error {:?}", e))
}


#[test]
fn sub_account_test() {
    let account_ad = 18_446_744_073_709_551_615u64 as u64;
    let a = id_to_subaccount(account_ad);
    // print!("{}", a)
}

export_service!();

#[ic_cdk_macros::query(name = "__get_candid_interface")]
fn export_candid() -> String {
    __export_service()
}