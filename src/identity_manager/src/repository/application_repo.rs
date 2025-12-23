use candid::{CandidType, Deserialize};
use std::cmp::Ordering;
use std::collections::HashSet;

use crate::repository::repo::APPLICATIONS;

#[deprecated()]
#[derive(Clone, Debug, CandidType, Deserialize, Eq)]
pub struct Application {
    pub domain: String,
    pub user_limit: u16,
    pub alias: Option<HashSet<String>>,
    pub img: Option<String>,
    pub name: String,
    pub is_nft_storage: Option<bool>,
    pub is_trusted: Option<bool>,
    pub is_iframe_allowed: Option<bool>,
}

impl PartialEq for Application {
    fn eq(&self, other: &Self) -> bool {
        self.domain.eq(&other.domain)
    }
}

impl Ord for Application {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.domain.cmp(&other.domain)
    }
}

impl PartialOrd<Self> for Application {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub trait ApplicationRepoTrait {
    fn read_applications(&self) -> Vec<Application>;
}

#[derive(Default)]
pub struct ApplicationRepo {}

impl ApplicationRepoTrait for ApplicationRepo {
    fn read_applications(&self) -> Vec<Application> {
        APPLICATIONS.with(|apps| {
            let applications = apps.borrow();
            applications.iter().cloned().collect()
        })
    }
}
