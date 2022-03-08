use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use crate::{AccountRepo};
use crate::repository::account_repo::{Account, AccountRepoTrait};
use ic_cdk::export::candid::{CandidType, Deserialize};
use serde_bytes::ByteBuf;
use crate::repository::repo::BasicEntity;


#[derive(Clone, Debug, CandidType, Deserialize, Eq)]
pub struct AccessPoint {
    pub pub_key: ByteBuf,
    pub base_fields: BasicEntity,
}

impl PartialEq for AccessPoint {
    fn eq(&self, other: &Self) -> bool {
        self.pub_key == other.pub_key
    }
}

impl Hash for AccessPoint {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pub_key.hash(state)
    }
}

pub trait AccessPointRepoTrait {
    fn get_access_points(&self) -> Option<HashSet<AccessPoint>>;
    fn store_access_points(&self, access_points: HashSet<AccessPoint>) -> Option<Account>;
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

    fn store_access_points(&self, access_points: HashSet<AccessPoint>) -> Option<Account> {
        let mut acc = self.account_repo.get_account()
            .unwrap().clone();
        acc.access_points = access_points.clone();
        let resp = self.account_repo.store_account(acc);
        match &resp {
            None => {}
            Some(r) => {
                access_points.into_iter().map(|ap| self.account_repo.update_account_index_with_pub_key(ap.pub_key));
            }
        };
        resp
    }
}