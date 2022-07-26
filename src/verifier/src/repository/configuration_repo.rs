use std::time::Duration;
use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use ic_cdk::storage;

#[derive(Debug, Deserialize, CandidType, Clone)]
pub struct Configuration {
    pub identity_manager_canister_id: String,
    pub whitelisted_canisters: Option<Vec<Principal>>,
    pub token_ttl: Duration,
}

pub struct ConfigurationRepo {}

pub struct AdminRepo {}

pub struct ControllersRepo {}


impl ConfigurationRepo {
    //todo fix Principle not implement default!
    pub fn get() -> &'static Configuration {
        storage::get::<Option<Configuration>>().as_ref().unwrap()
    }

    pub fn exists() -> bool {
        storage::get::<Option<Configuration>>().is_some()
    }

    pub fn save(configuration: Configuration) -> () {
        storage::get_mut::<Option<Configuration>>().replace(configuration);
    }
}


impl AdminRepo {
    pub fn get() -> Principal {
        storage::get_mut::<Option<Principal>>().unwrap()
    }

    pub fn save(principal: Principal) -> () {
        storage::get_mut::<Option<Principal>>().replace(principal);
    }
}

impl ControllersRepo {
    pub fn get() -> Vec<Principal> {
        storage::get_mut::<Vec<Principal>>().to_vec()
    }

    pub fn save(principals: Vec<Principal>) -> () {
        let vec = storage::get_mut::<Vec<Principal>>();
        vec.clear();
        vec.extend(principals);
    }

    pub fn contains(principal: &Principal) -> bool {
        storage::get_mut::<Vec<Principal>>().to_vec().contains(principal)
    }
}