use ic_cdk::storage;

use ic_cdk::export::candid::{CandidType, Deserialize};

#[derive(Debug, Deserialize, CandidType, Clone)]
pub struct Configuration {
    pub identity_manager_canister_id: String,
}

pub struct ConfigurationRepo {}

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