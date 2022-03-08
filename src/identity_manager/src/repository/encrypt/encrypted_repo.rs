use std::collections::{BTreeMap, HashSet};
use std::hash::{Hash, Hasher};

use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use ic_cdk::storage;
use serde_bytes::ByteBuf;

use crate::repository::account_repo::Account;
use crate::repository::encrypt::account_encrypt::{decrypt_account, encrypt, encrypt_account};
use crate::repository::repo::{BasicEntity, is_anchor_exists};
use crate::service::ic_service;

pub type EncryptedAccounts = BTreeMap<String, EncryptedAccount>;
pub type PrincipalIndex = BTreeMap<String, String>;

#[derive(Default, Clone, Debug, CandidType, Deserialize, Eq)]
pub struct EncryptedAccessPoint {
    pub pub_key: ByteBuf,
    pub base_fields: BasicEntity,
}

impl PartialEq for EncryptedAccessPoint {
    fn eq(&self, other: &Self) -> bool {
        self.pub_key == other.pub_key
    }
}

impl Hash for EncryptedAccessPoint {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pub_key.hash(state)
    }
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct EncryptedPersona {
    pub anchor: Option<String>,
    pub domain: String,
    pub persona_id: Option<String>,
    pub base_fields: BasicEntity,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct EncryptedAccount {
    pub anchor: String,
    pub principal_id: String,
    pub name: Option<String>,
    pub phone_number: Option<String>,
    pub personas: Vec<EncryptedPersona>,
    pub access_points: HashSet<EncryptedAccessPoint>,
    pub base_fields: BasicEntity,
}

pub struct EncryptedRepo {}

impl EncryptedRepo {
    pub fn create_account(account: Account) -> Option<Account> {
        let accounts = storage::get_mut::<EncryptedAccounts>();
        let index = storage::get_mut::<PrincipalIndex>();
        if is_anchor_exists(account.anchor) {
            None
        } else {
            let encr_acc = encrypt_account(account.clone());
            index.insert(encr_acc.principal_id.clone(), encr_acc.principal_id.clone());
            accounts.insert(encr_acc.principal_id.clone(), encr_acc);
            Some(account)
        }
    }

    pub fn store_account(account: Account) -> Option<Account> {
        let accounts = storage::get_mut::<EncryptedAccounts>();
        let encr_acc = encrypt_account(account.clone());
        accounts.insert(encr_acc.principal_id.clone(), encr_acc);
        Some(account)
    }

    pub fn get_account() -> Option<Account> {
        let princ = ic_service::get_caller().to_text();

        let accounts = storage::get_mut::<EncryptedAccounts>();
        match accounts.get(&encrypt(princ.to_owned())) {
            None => { None }
            Some(acc) => { Option::from(decrypt_account(acc.to_owned())) }
        }
    }

    pub fn remove_account() -> Option<Account> {
        let princ = ic_service::get_caller().to_text();
        let accounts = storage::get_mut::<EncryptedAccounts>();
        match accounts.remove(&encrypt(princ.to_owned())) {
            None => { None }
            Some(acc) => { Option::from(decrypt_account(acc.to_owned())) }
        }
    }

    pub fn update_account_index_with_pub_key(additional_key: ByteBuf){
        let princ = ic_service::get_caller().to_text();
        let key = Principal::self_authenticating(additional_key);
        let index = storage::get_mut::<PrincipalIndex>();
        index.insert(encrypt(key.to_text()), encrypt(princ));
    }

    pub fn exists(principal: &Principal) -> bool {
        storage::get::<EncryptedAccounts>().contains_key(&encrypt(principal.to_text()))
    }
}

