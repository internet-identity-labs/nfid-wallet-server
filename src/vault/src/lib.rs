extern crate core;

use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

use candid::{candid_method, export_service, Principal};
use candid::utils::ArgumentEncoder;
use ic_cdk::{caller, storage, trap};
use ic_cdk::export::{candid::{CandidType, Deserialize}};
use ic_cdk::export::candid;
use ic_cdk_macros::*;
use ic_cdk_macros::*;
use ic_ledger_types::{AccountIdentifier, BlockIndex, DEFAULT_FEE, MAINNET_LEDGER_CANISTER_ID, Memo, Subaccount, Tokens};

use crate::policy_service::Policy;
use crate::transaction_service::Transaction;
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


#[derive(CandidType, Deserialize, Clone)]
pub struct VaultRegisterRequest {
    name: String,
}

#[update]
#[candid_method(update)]
async fn register_vault(request: VaultRegisterRequest) -> Vault {
    let address = caller_to_address();
    let mut user = user_service::get_or_new_by_address(address);
    let vault = vault_service::register(user.address.clone(), request.name);
    user.vaults.push(vault.id.clone());
    user_service::restore(user);
    vault
}

#[query]
#[candid_method(query)]
async fn get_vaults() -> Vec<Vault> {
    let vault_ids = user_service::get_or_new_by_caller().vaults;
    vault_service::get_by_ids(vault_ids)
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
    let caller = get_or_new_by_caller();
    VAULTS.with(|vaults| {
        return match vaults.borrow_mut().get_mut(&request.vault_id) {
            None => { trap("Vault not exists") }
            Some(vault) => {
                let caller_member = vault.members
                    .iter()
                    .find(|l| l.user_uuid.eq(&caller.address));
                match caller_member {
                    None => {
                        trap("Unauthorised")
                    }
                    Some(member) => {
                        match member.role {
                            VaultRole::Admin => {
                                let mut user = user_service::get_or_new_by_address(request.address);
                                let vm = VaultMember {
                                    user_uuid: user.address.clone(),
                                    role: request.role,
                                    name: request.name,
                                };
                                vault.members.insert(vm);
                                user.vaults.push(vault.id.clone());
                                user_service::restore(user);
                                vault.clone()
                            }
                            VaultRole::Member => {
                                trap("Not enough permissions")
                            }
                        }
                    }
                }
            }
        };
    })
}

#[query]
#[candid_method(query)]
async fn get_vault_members(vault_id: u64) -> HashSet<VaultMember> {
    let caller = get_or_new_by_caller();
    if !caller.vaults.contains(&vault_id) {
        trap("Not participant")
    }
    VAULTS.with(|vaults| {
        return match vaults.borrow_mut().get_mut(&vault_id) {
            None => { trap("Vault not exists") }
            Some(vault) => {
                vault.members.clone()
            }
        };
    })
}

#[derive(CandidType, Deserialize, Clone)]
pub struct WalletRegisterRequest {
    vault_id: u64,
    name: Option<String>,
}

#[update]
#[candid_method(update)]
async fn register_wallet(request: WalletRegisterRequest) -> Wallet {
    let caller = get_or_new_by_caller();
    let mut vault = match vault_service::get_by_ids(vec![request.vault_id]).first() {
        None => { trap("Vault not exists") }
        Some(vault) => {
            let caller_member = vault.members
                .iter()
                .find(|p| caller.address.eq(&p.user_uuid));
            match caller_member {
                None => {
                    trap("Unauthorised")
                }
                Some(vaultMember) => {
                    match vaultMember.role {
                        VaultRole::Admin => {
                            vault.clone()
                        }
                        VaultRole::Member => {
                            trap("Not enough permissions")
                        }
                    }
                }
            }
        }
    };
    let new_wallet = wallet_service::new_and_store(request.name, request.vault_id);
    vault.wallets.push(new_wallet.id);
    vault_service::restore(vault);
    new_wallet
}

#[query]
#[candid_method(query)]
async fn get_wallets(vault_id: u64) -> Vec<Wallet> {
    let caller = get_or_new_by_caller();
    if !caller.vaults.contains(&vault_id) {
        trap("Not participant")
    }
    let vault = vault_service::get_by_ids(vec![vault_id]);
    wallet_service::get_wallets(vault[0].wallets.clone())
}


#[update]
async fn register_transaction(amount: Tokens, to: AccountIdentifier, wallet_id: u64) -> Transaction {
    let caller = caller().to_text();
    let tr_owner = user_service::get_by_address(caller);
    let mut wallet = wallet_service::get_wallet(wallet_id);
    let vaults = vault_service::get_by_ids(wallet.vaults.clone());
    let vault = vaults.first().unwrap(); //one to one??

    let user_id = tr_owner.address.clone();

    let policy = policy_service::get_by_ids(vault.policies.clone());


    let transaction = transaction_service::register_transaction(amount, to, wallet_id, tr_owner, policy.first().unwrap().clone());

    // wallet.transaction_ids.push(transaction.id);//todo think about index
    // wallet_service::restore(wallet);

    let users_to_notify = vault.members.clone().into_iter()
        .map(|k| k.user_uuid)
        .filter(|l| !l.eq(&user_id))
        .collect();

    notification_service::register_notification(transaction.id, users_to_notify);
    transaction
}


#[update]
async fn approve_transaction(transaction_id: u64) -> Transaction {
    let caller = caller().to_text();
    let tr_owner = user_service::get_by_address(caller);
    let mut transaction = transaction_service::approve_transaction(transaction_id, tr_owner);
    if transaction.approves.len() as u8 >= transaction.member_threshold {
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


#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct VaultMemoryObject {
    pub vaults: Vec<Vault>,
    pub users: Vec<User>,
    pub wallets: Vec<Wallet>,
}

#[pre_upgrade]
fn pre_upgrade() {
    let vv: Vec<Vault> = VAULTS.with(|vaults| {
        vaults.borrow()
            .iter()
            .map(|l| l.1.clone())
            .collect()
    });
    let ww: Vec<Wallet> = WALLETS.with(|wallets| {
        wallets.borrow()
            .iter()
            .map(|l| l.1.clone())
            .collect()
    });
    let uu: Vec<User> = USERS.with(|users| {
        users.borrow()
            .iter()
            .map(|l| l.1.clone())
            .collect()
    });
    let memory = VaultMemoryObject {
        vaults: vv,
        users: uu,
        wallets: ww,
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
    for u in mo.users {
        users.insert(u.address.clone(), u);
    }
    VAULTS.with(|storage| *storage.borrow_mut() = vaults);
    USERS.with(|storage| *storage.borrow_mut() = users);
    WALLETS.with(|storage| *storage.borrow_mut() = wallets);
}
