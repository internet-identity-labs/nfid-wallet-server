use crate::http::requests::WalletVariant;
use crate::ic_service;
use crate::repository::access_point_repo::AccessPoint;
use crate::repository::persona_repo::Persona;
use crate::repository::repo::{is_anchor_exists, BasicEntity};
use crate::service::certified_service::{remove_certify_keys, update_certify_keys};
use candid::{CandidType, Deserialize, Principal};
use ic_cdk::storage;
use itertools::Itertools;
use serde::Serialize;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashSet};

thread_local! {
  pub static ACCOUNTS: RefCell<BTreeMap<String, Account>> = RefCell::new(BTreeMap::new());
  pub static PRINCIPAL_INDEX: RefCell<BTreeMap<String, String>> = RefCell::new(BTreeMap::new());
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Account {
    pub anchor: u64,
    pub principal_id: String,
    #[deprecated()]
    pub name: Option<String>,
    #[deprecated()]
    pub phone_number: Option<String>,
    #[deprecated()]
    pub phone_number_sha2: Option<String>,
    #[deprecated()]
    pub personas: Vec<Persona>,
    pub access_points: HashSet<AccessPoint>,
    pub base_fields: BasicEntity,
    pub wallet: WalletVariant,
    pub is2fa_enabled: bool,
    pub email: Option<String>,
}

pub trait AccountRepoTrait {
    fn get_account(&self) -> Option<Account>;
    fn get_account_by_principal(&self, princ: String) -> Option<Account>;
    fn get_account_by_anchor(&self, anchor: u64, wallet: WalletVariant) -> Option<Account>;
    fn create_account(&self, account: Account) -> Option<Account>;
    fn store_account(&self, account: Account) -> Option<Account>;
    fn remove_account(&self) -> Option<Account>;
    fn exists(&self, principal: &Principal) -> bool;
    fn update_account_index_with_pub_key(&self, additional_key: String, princ: String);
    fn update_account_index(&self, additional_principal_id: String);
    fn remove_account_index(&self, additional_principal_id: String);
    fn get_all_accounts(&self) -> Vec<Account>;
    fn find_next_nfid_anchor(&self) -> u64;
    fn reset_2fa(&self);
}

#[derive(Default, Clone, Copy)]
pub struct AccountRepo {}

impl AccountRepoTrait for AccountRepo {
    fn get_account(&self) -> Option<Account> {
        let princ = ic_service::get_caller().to_text();
        PRINCIPAL_INDEX.with(|index| {
            ACCOUNTS.with(|accounts| match index.borrow().get(&princ) {
                None => None,
                Some(key) => match accounts.borrow().get(key) {
                    None => None,
                    Some(acc) => Option::from(acc.to_owned()),
                },
            })
        })
    }

    fn get_account_by_principal(&self, princ: String) -> Option<Account> {
        PRINCIPAL_INDEX.with(|index| {
            ACCOUNTS.with(|accounts| match index.borrow().get(&princ) {
                None => None,
                Some(key) => match accounts.borrow().get(key) {
                    None => None,
                    Some(acc) => Option::from(acc.to_owned()),
                },
            })
        })
    }

    fn get_account_by_anchor(&self, anchor: u64, wallet: WalletVariant) -> Option<Account> {
        ACCOUNTS.with(|accounts| {
            match accounts
                .borrow()
                .iter()
                .find(|l| l.1.anchor == anchor && l.1.wallet == wallet)
            {
                None => None,
                Some(pair) => Some(pair.1.to_owned()),
            }
        })
    }

    fn create_account(&self, account: Account) -> Option<Account> {
        ACCOUNTS.with(|accounts| {
            PRINCIPAL_INDEX.with(|index| {
                if index.borrow().contains_key(&account.principal_id) {
                    return None;
                }
                if is_anchor_exists(account.anchor, account.wallet.clone()) {
                    return None;
                } else {
                    index
                        .borrow_mut()
                        .insert(account.principal_id.clone(), account.principal_id.clone());
                    update_certify_keys(account.principal_id.clone(), account.principal_id.clone());
                    accounts
                        .borrow_mut()
                        .insert(account.principal_id.clone(), account.clone());
                    Some(account)
                }
            })
        })
    }

    fn store_account(&self, account: Account) -> Option<Account> {
        ACCOUNTS.with(|accounts| {
            accounts
                .borrow_mut()
                .insert(account.principal_id.clone(), account.clone());
            Some(account)
        })
    }

    fn remove_account(&self) -> Option<Account> {
        //todo not properly tested, used for e2e tests
        self.get_account(); //security call
        let princ = ic_service::get_caller().to_text();
        PRINCIPAL_INDEX.with(|index| {
            ACCOUNTS.with(|accounts| match accounts.borrow_mut().remove(&princ) {
                None => None,
                Some(acc) => {
                    (&acc.access_points).into_iter().for_each(|ap| {
                        index.borrow_mut().remove(&ap.principal_id);
                    });
                    index.borrow_mut().remove(&acc.principal_id.clone());
                    remove_certify_keys(acc.principal_id.clone());
                    Option::from(acc.to_owned())
                }
            })
        })
    }

    fn exists(&self, principal: &Principal) -> bool {
        PRINCIPAL_INDEX.with(|index| index.borrow().contains_key(&principal.to_text()))
    }

    fn update_account_index_with_pub_key(&self, additional_principal_id: String, princ: String) {
        PRINCIPAL_INDEX.with(|index| {
            update_certify_keys(additional_principal_id.clone(), princ.clone());
            index
                .borrow_mut()
                .insert(additional_principal_id.clone(), princ.clone());
        })
    }

    fn update_account_index(&self, additional_principal_id: String) {
        PRINCIPAL_INDEX.with(|index| {
            update_certify_keys(
                additional_principal_id.clone(),
                additional_principal_id.clone(),
            );
            index.borrow_mut().insert(
                additional_principal_id.clone(),
                additional_principal_id.clone(),
            );
        })
    }

    fn remove_account_index(&self, additional_principal_id: String) {
        PRINCIPAL_INDEX.with(|index| {
            remove_certify_keys(additional_principal_id.clone());
            index.borrow_mut().remove(&additional_principal_id);
        })
    }

    fn get_all_accounts(&self) -> Vec<Account> {
        ACCOUNTS.with(|accounts| accounts.borrow().values().map(|l| l.to_owned()).collect())
    }

    fn find_next_nfid_anchor(&self) -> u64 {
        ACCOUNTS.with(|accounts| {
            accounts
                .borrow()
                .values()
                .filter(|a| a.wallet.eq(&WalletVariant::NFID))
                .sorted_by(|a, b| Ord::cmp(&a.anchor, &b.anchor))
                .last()
                .map_or(100_000_000, |x| {
                    if x.anchor < 200_000_000 {
                        200_000_000
                    } else {
                        x.anchor + 1
                    }
                })
        })
    }


    fn reset_2fa(&self)  {
        ACCOUNTS.with(|accounts| accounts.borrow_mut().values_mut().for_each(|acc| acc.is2fa_enabled = false));
    }
}
