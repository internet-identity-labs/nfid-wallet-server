use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use crate::{AccountRepo};
use crate::repository::account_repo::{Account, AccountRepoTrait};
use ic_cdk::export::candid::{CandidType, Deserialize};
use crate::repository::repo::BasicEntity;
use serde::{Serialize};


#[derive(Clone, Debug, CandidType, Deserialize, Eq, Serialize)]
pub struct AccessPoint {
    pub principal_id: String,
    pub icon: Option<String>,
    pub device: Option<String>,
    pub browser: Option<String>,
    pub last_used: Option<u64>,
    pub base_fields: BasicEntity,
}

impl PartialEq for AccessPoint {
    fn eq(&self, other: &Self) -> bool {
        self.principal_id == other.principal_id
    }
}

impl Hash for AccessPoint {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.principal_id.hash(state)
    }
}

pub trait AccessPointRepoTrait {
    fn get_access_points(&self) -> Option<HashSet<AccessPoint>>;
    fn get_access_points_by_principal(&self, princ: String) -> Option<HashSet<AccessPoint>>;
    fn use_access_point(&self, ap_principal: String, time: u64) -> Option<AccessPoint>;
    fn store_access_points(&self, access_points: HashSet<AccessPoint>) -> Option<Account>;
    fn store_access_points_by_principal(&self, access_points: HashSet<AccessPoint>, root_princ: String) -> Option<Account>;
    fn update_account_index(&self, additional_principal_id: String, root_princ: String);
}

#[derive(Default)]
pub struct AccessPointRepo {
    pub account_repo: AccountRepo,
}

impl AccessPointRepoTrait for AccessPointRepo {
    fn get_access_points(&self) -> Option<HashSet<AccessPoint>> {
        self.account_repo.get_account()
            .map(|x| x.access_points.clone()) //todo &
    }

    fn get_access_points_by_principal(&self, princ: String) -> Option<HashSet<AccessPoint>> {
        self.account_repo.get_account_by_principal(princ)
            .map(|x| x.access_points.clone()) //todo &
    }

    fn use_access_point(&self, ap_principal: String, time: u64) -> Option<AccessPoint> {
        let mut points = self.get_access_points().unwrap();
        let updated = points.clone()
            .into_iter()
            .find(|l| l.principal_id == ap_principal);
        match updated {
            None => { None }
            Some(mut ap) => {
                ap.last_used = Some(time);
                points.replace(ap.clone());
                self.store_access_points(points);
                Some(ap)
            }
        }
    }

    fn store_access_points(&self, access_points: HashSet<AccessPoint>) -> Option<Account> {
        let mut acc = self.account_repo.get_account()
            .unwrap().clone();
        acc.access_points = access_points.clone();
        let resp = self.account_repo.store_account(acc);
        resp
    }

    fn store_access_points_by_principal(&self, access_points: HashSet<AccessPoint>, root_princ: String) -> Option<Account> {
        let mut acc = self.account_repo.get_account_by_principal(root_princ)
            .unwrap().clone();
        acc.access_points = access_points.clone();
        let resp = self.account_repo.store_account(acc);
        resp
    }

    fn update_account_index(&self, additional_principal_id: String, root_princ: String) {
        self.account_repo.update_account_index_with_pub_key(additional_principal_id, root_princ)
    }
}