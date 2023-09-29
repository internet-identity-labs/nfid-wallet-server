use std::collections::{BTreeMap, HashSet};
#[cfg(test)]
use mockers_derive::mocked;
use crate::repository::persona_repo::Persona;
use crate::repository::repo::{BasicEntity, is_anchor_exists};
use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use ic_cdk::{storage};
use itertools::Itertools;
use crate::{ic_service};
use crate::repository::access_point_repo::AccessPoint;
use serde::{Serialize};
use crate::http::requests::{WalletVariant};
use crate::service::certified_service::{remove_certify_keys, update_certify_keys};

pub type Accounts = BTreeMap<String, Account>;
pub type PrincipalIndex = BTreeMap<String, String>;

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Account {
    pub anchor: u64,
    pub principal_id: String,
    pub name: Option<String>,
    pub phone_number: Option<String>,
    pub phone_number_sha2: Option<String>,
    pub personas: Vec<Persona>,
    pub access_points: HashSet<AccessPoint>,
    pub base_fields: BasicEntity,
    pub wallet: WalletVariant,
    pub is2fa_enabled: bool,
    pub email: Option<String>,
}

#[cfg_attr(test, mocked)]
pub trait AccountRepoTrait {
    fn get_account(&self) -> Option<Account>;
    fn get_account_by_principal(&self, princ: String) -> Option<Account>;
    fn get_account_by_anchor(&self, anchor: u64, wallet: WalletVariant) -> Option<Account>;
    fn create_account(&self, account: Account) -> Option<Account>;
    fn store_account(&self, account: Account) -> Option<Account>;
    fn remove_account(&self) -> Option<Account>;
    fn remove_account_by_principal(&self, princ: String) -> Option<Account>;
    fn exists(&self, principal: &Principal) -> bool;
    fn update_account_index_with_pub_key(&self, additional_key: String, princ: String);
    fn update_account_index(&self, additional_principal_id: String);
    fn remove_account_index(&self, additional_principal_id: String);
    fn get_accounts(&self, ids: Vec<String>) -> Vec<Account>;
    fn get_all_accounts(&self) -> Vec<Account>;
    fn find_next_nfid_anchor(&self) -> u64;
    fn store_accounts(&self, accounts: Vec<Account>);
    fn get_account_by_id(&self, princ: String) -> Option<Account>;
}

#[derive(Default, Clone, Copy)]
pub struct AccountRepo {}

impl AccountRepoTrait for AccountRepo {
    fn get_account(&self) -> Option<Account> {
        let princ = ic_service::get_caller().to_text();
        let index = storage::get_mut::<PrincipalIndex>();
        let accounts = storage::get_mut::<Accounts>();
        match index.get_mut(&princ) {
            None => { None }
            Some(key) => {
                match accounts.get(key) {
                    None => { None }
                    Some(acc) => { Option::from(acc.to_owned()) }
                }
            }
        }
    }

    fn get_account_by_principal(&self, princ: String) -> Option<Account> {
        let index = storage::get_mut::<PrincipalIndex>();
        let accounts = storage::get_mut::<Accounts>();
        match index.get_mut(&princ) {
            None => { None }
            Some(key) => {
                match accounts.get(key) {
                    None => { None }
                    Some(acc) => {
                        // verify_2fa(acc, princ);
                        Option::from(acc.to_owned())
                    }
                }
            }
        }
    }

    fn get_account_by_anchor(&self, anchor: u64, wallet: WalletVariant) -> Option<Account> {
        let accounts = storage::get_mut::<Accounts>();
        match accounts.iter()
            .find(|l| l.1.anchor == anchor && l.1.wallet == wallet) {
            None => { None }
            Some(pair) => {
                Some(pair.1.to_owned())
            }
        }
    }

    fn create_account(&self, account: Account) -> Option<Account> {
        let accounts = storage::get_mut::<Accounts>();
        let index = storage::get_mut::<PrincipalIndex>();
        if index.contains_key(&account.principal_id) {
            return None;
        }
        if is_anchor_exists(account.anchor, account.wallet.clone()) {
            return None;
        } else {
            index.insert(account.principal_id.clone(), account.principal_id.clone());
            update_certify_keys(account.principal_id.clone(), account.principal_id.clone());
            accounts.insert(account.principal_id.clone(), account.clone());
            Some(account)
        }
    }

    fn store_account(&self, account: Account) -> Option<Account> {
        let accounts = storage::get_mut::<Accounts>();
        accounts.insert(account.principal_id.clone(), account.clone());
        Some(account)
    }

    fn remove_account(&self) -> Option<Account> { //todo not properly tested, used for e2e tests
        self.get_account(); //security call
        let princ = ic_service::get_caller().to_text();
        let accounts = storage::get_mut::<Accounts>();
        let index = storage::get_mut::<PrincipalIndex>();
        match accounts.remove(&princ) {
            None => { None }
            Some(acc) => {
                (&acc.access_points).into_iter()
                    .for_each(|ap| { index.remove(&ap.principal_id); });
                index.remove(&acc.principal_id.clone());
                remove_certify_keys(acc.principal_id.clone());
                Option::from(acc.to_owned())
            }
        }
    }

    fn remove_account_by_principal(&self, princ: String) -> Option<Account> {
        let accounts = storage::get_mut::<Accounts>();
        let index = storage::get_mut::<PrincipalIndex>();
        match accounts.remove(&princ) {
            None => { None }
            Some(acc) => {
                (&acc.access_points).into_iter()
                    .for_each(|ap| { index.remove(&ap.principal_id); });
                index.remove(&acc.principal_id.clone());
                Option::from(acc.to_owned())
            }
        }
    }

    fn exists(&self, principal: &Principal) -> bool {
        storage::get_mut::<PrincipalIndex>().contains_key(&principal.to_text())
    }

    fn update_account_index_with_pub_key(&self, additional_principal_id: String, princ: String) {
        let index = storage::get_mut::<PrincipalIndex>();
        update_certify_keys(additional_principal_id.clone(), princ.clone());
        index.insert(additional_principal_id, princ);
    }

    fn update_account_index(&self, additional_principal_id: String) {
        let index = storage::get_mut::<PrincipalIndex>();
        update_certify_keys(additional_principal_id.clone(), additional_principal_id.clone());
        index.insert(additional_principal_id.clone(), additional_principal_id);
    }

    fn remove_account_index(&self, additional_principal_id: String) {
        let index = storage::get_mut::<PrincipalIndex>();
        remove_certify_keys(additional_principal_id.clone());
        index.remove( &additional_principal_id);
    }

    fn get_accounts(&self, ids: Vec<String>) -> Vec<Account> {
        let index = storage::get_mut::<PrincipalIndex>();
        let accounts = storage::get_mut::<Accounts>();
        ids.into_iter()
            .map(|i| index.get(&i))
            .filter(|l| l.is_some())
            .map(|i| i.unwrap())
            .map(|i| accounts.get(i))
            .filter(|l| l.is_some())
            .map(|i| i.unwrap().to_owned())
            .unique_by(|p| p.to_owned().principal_id)
            .map(|l| l.to_owned())
            .collect::<Vec<_>>()
    }

    fn get_all_accounts(&self) -> Vec<Account> {
        storage::get_mut::<Accounts>()
            .values()
            .map(|l| l.to_owned())
            .collect()
    }

    fn find_next_nfid_anchor(&self) -> u64 {
        let acc = self.get_all_accounts()
            .into_iter()
            .filter(|a| a.wallet.eq(&WalletVariant::NFID))
            .sorted_by(|a, b| Ord::cmp(&a.anchor, &b.anchor))
            .last();

        match acc {
            None => { 100_000_000 }
            Some(x) => {
                x.anchor + 1
            }
        }
    }

    fn store_accounts(&self, accounts: Vec<Account>) {
        let accounts_stored = storage::get_mut::<Accounts>();
        for account in accounts {
            accounts_stored.insert(account.principal_id.clone(), account.clone());
            self.update_account_index(account.principal_id.clone());
        }
    }

    fn get_account_by_id(&self, princ: String) -> Option<Account> {
        let index = storage::get_mut::<PrincipalIndex>();
        let accounts = storage::get_mut::<Accounts>();
        match index.get_mut(&princ) {
            None => { None }
            Some(key) => {
                match accounts.get(key) {
                    None => { None }
                    Some(acc) => { Option::from(acc.to_owned()) }
                }
            }
        }
    }
}
