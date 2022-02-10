use crate::repository::encrypt::encrypted_repo::EncryptedRepo;
use crate::repository::repo::Account;
#[cfg(test)]
use mockers_derive::mocked;

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