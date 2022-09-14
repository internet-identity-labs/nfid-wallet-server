use std::cmp::Ordering;
use std::collections::HashSet;
use ic_cdk::{storage};

use crate::repository::repo::Applications;
use ic_cdk::export::candid::{CandidType, Deserialize};

#[derive(Clone, Debug, CandidType, Deserialize, Eq)]
pub struct Application {
    pub domain: String,
    pub user_limit: u16,
    pub alias: Option<HashSet<String>>,
    pub img: Option<String>,
    pub name: String,
    pub is_nft_storage: Option<bool>
}

impl PartialEq for Application{
    fn eq(&self, other: &Self) -> bool {
        self.domain == other.domain
    }
}

impl Ord for Application {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        return *&self.name.cmp(&other.name);
    }
}

impl PartialOrd<Self> for Application {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub trait ApplicationRepoTrait {
    fn create_application(&self, application: Application) -> Vec<Application>;
    fn read_applications(&self) -> Vec<Application>;
    fn update_application(&self, app: Application) -> bool;
    fn delete_application(&self, domain: String) -> bool;
    fn is_application_exists(&self, application: &Application) -> bool;
    fn get_application(&self, domain: &String) -> Option<&Application>;
}

#[derive(Default)]
pub struct ApplicationRepo {}

impl ApplicationRepoTrait for ApplicationRepo {
    fn create_application(&self, application: Application) -> Vec<Application> {
        let applications = storage::get_mut::<Applications>();
        applications.insert(application.clone());
        applications.iter()
            .map(|p| p.clone())
            .collect()
    }

    fn read_applications(&self) -> Vec<Application> {
        storage::get_mut::<Applications>().iter()
            .map(|p| p.clone())
            .collect()
    }

    fn update_application(&self, app: Application) -> bool {
        storage::get_mut::<Applications>().replace(app).is_some()
    }

    fn delete_application(&self, domain: String) -> bool {
        let app_to_remove = storage::get_mut::<Applications>().iter()
            .find(|a| a.domain.eq(&domain));
        match app_to_remove {
            None => { false }
            Some(app) => {
                storage::get_mut::<Applications>()
                    .remove(app)
            }
        }
    }

    fn is_application_exists(&self, application: &Application) -> bool {
        let applications = storage::get_mut::<Applications>();
        applications.iter()
            .any(|a|a.domain.eq(&application.domain))
    }

    fn get_application(&self, domain: &String) -> Option<&Application> {
        let applications = storage::get_mut::<Applications>();
        applications.iter()
            .find(|a|  a.domain.eq(domain))
    }
}