use crate::repository::encrypt::encrypted_repo::{EncryptedRepo};
#[cfg(test)]
use mockers_derive::mocked;
use crate::repository::persona_repo::Persona;
use crate::repository::repo::BasicEntity;
use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Account {
    pub anchor: u64,
    pub principal_id: String,
    pub name: Option<String>,
    pub phone_number: Option<String>,
    pub personas: Vec<Persona>,
    pub base_fields: BasicEntity,
}

#[cfg_attr(test, mocked)]
pub trait AccountRepoTrait {
    fn get_account(&self) -> Option<Account>;
    fn create_account(&self, account: Account) -> Option<Account>;
    fn store_account(&self, account: Account) -> Option<Account>;
    fn remove_account(&self) -> Option<Account>;
    fn exists(&self, principal: &Principal) -> bool;
}

#[derive(Default)]
pub struct AccountRepo {}

impl AccountRepoTrait for AccountRepo {
    fn get_account(&self) -> Option<Account> {
        EncryptedRepo::get_account()
    }

    fn create_account(&self, account: Account) -> Option<Account> {
        EncryptedRepo::create_account(account)
    }

    fn store_account(&self, account: Account) -> Option<Account> {
        EncryptedRepo::store_account(account)
    }

    fn remove_account(&self) -> Option<Account> {
        EncryptedRepo::remove_account()
    }

    fn exists(&self, principal: &Principal) -> bool {
        EncryptedRepo::exists(principal)
    }
}
