use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::storage;


use crate::repository::repo::{Account, EncryptedAccounts, is_anchor_exists};
use crate::repository::encrypt::account_encrypt::{decrypt_account, encrypt, encrypt_account};
use crate::service::ic_service;

#[derive(Default, Clone, Debug, CandidType, Deserialize, PartialEq)]
pub struct EncryptedAccessPoint {
    pub pub_key: String,
    pub last_used: String,
    pub make: String,
    pub model: String,
    pub browser: String,
    pub name: String,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct EncryptedPersona {
    pub anchor: Option<String>,
    pub domain: String,
    pub persona_id: Option<String>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct EncryptedAccount {
    pub anchor: String,
    pub principal_id: String,
    pub name: String,
    pub phone_number: String,
    pub access_points: Vec<EncryptedAccessPoint>,
    pub personas: Vec<EncryptedPersona>,
}

pub struct EncryptedRepo {}

impl EncryptedRepo {
    pub fn create_account(account: Account) -> Option<Account> {
        let accounts = storage::get_mut::<EncryptedAccounts>();
        if is_anchor_exists(account.anchor) {
            None
        } else {
            let encr_acc = encrypt_account(account.clone());
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
}

