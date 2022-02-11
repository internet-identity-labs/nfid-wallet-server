use std::collections::HashSet;
use crate::repository::encrypt::encrypted_repo::EncryptedRepo;
#[cfg(test)]
use mockers_derive::mocked;
use crate::repository::access_point_repo::AccessPoint;
use crate::repository::persona_repo::Persona;
use crate::repository::repo::BasicEntity;
use ic_cdk::export::candid::{CandidType, Deserialize};

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Account {
    pub anchor: u64,
    pub principal_id: String,
    pub name: String,
    pub phone_number: String,
    pub access_points: HashSet<AccessPoint>,
    pub personas: Vec<Persona>,
    pub base_fields: BasicEntity,
}

#[cfg_attr(test, mocked)]
pub trait AccountRepoTrait {
    fn get_account(&self) -> Option<Account>;
    fn create_account(&self, account: Account) -> Option<Account>;
    fn store_account(&self, account: Account) -> Option<Account>;
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
}