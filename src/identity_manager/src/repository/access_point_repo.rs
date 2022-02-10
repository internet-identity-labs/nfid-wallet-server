use crate::{AccessPoint, AccountRepo};
use crate::repository::account_repo::AccountRepoTrait;
use crate::repository::repo::Account;

pub trait AccessPointRepoTrait {
    fn get_access_points(&self) -> Option<Vec<AccessPoint>>;
    fn store_access_points(&self, access_points: Vec<AccessPoint>) -> Option<Account>;
}

#[derive(Default)]
pub struct AccessPointRepo {
    pub account_repo: AccountRepo,
}

impl AccessPointRepoTrait for AccessPointRepo {
    fn get_access_points(&self) -> Option<Vec<AccessPoint>> {
        self.account_repo.get_account()
            .map(|x| x.access_points.clone()) //todo &
    }

    fn store_access_points(&self, access_points: Vec<AccessPoint>) -> Option<Account> {
        let mut acc = self.account_repo.get_account()
            .unwrap().clone();
        acc.access_points = access_points;
        self.account_repo.store_account(acc)
    }
}