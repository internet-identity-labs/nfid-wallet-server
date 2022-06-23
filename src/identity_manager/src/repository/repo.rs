use std::collections::{BTreeSet, HashSet};
use std::hash::{Hash};
use std::time::Duration;
use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use ic_cdk::storage;
use crate::{ic_service};
use crate::logger::logger::Logs;
use crate::repository::account_repo::{Account, Accounts, PrincipalIndex};
use crate::repository::application_repo::Application;

use crate::repository::phone_number_repo::{PhoneNumberRepo, PhoneNumberRepoTrait};


#[derive(Debug, Deserialize, CandidType, Clone)]
pub struct Configuration {
    pub lambda: Principal,
    pub token_ttl: Duration,
    pub token_refresh_ttl: Duration,
    pub whitelisted_phone_numbers: Vec<String>,
    pub heartbeat: Option<u32>,
    pub backup_canister_id: Option<String>,
    pub ii_canister_id: Principal,
    pub whitelisted_canisters: Option<Vec<Principal>>,
    pub env: Option<String>,
    pub git_branch: Option<String>,
    pub commit_hash: Option<String>,
}

//todo rethink visibility
pub type Applications = BTreeSet<Application>;

pub struct AdminRepo {}

pub struct ConfigurationRepo {}

#[derive(Clone, Debug, CandidType, Deserialize, Default, PartialEq, Eq, Copy, Hash)]
pub struct BasicEntity {
    created_date: u64,
    modified_date: u64,
}

impl BasicEntity {
    pub fn get_created_date(self) -> u64 {
        self.created_date
    }
    pub fn get_modified_date(self) -> u64 {
        self.modified_date
    }
    pub fn update_modified_date(mut self) -> u64 {
        self.modified_date = ic_service::get_time();
        self.modified_date
    }
    pub fn new() -> BasicEntity {
        BasicEntity { created_date: ic_service::get_time(), modified_date: ic_service::get_time() }
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


impl ConfigurationRepo {
    //todo fix Principle not implement default!
    pub fn get() -> &'static Configuration {
        if (storage::get::<Option<Configuration>>()).is_none() {
            ConfigurationRepo::save(ConfigurationRepo::get_default_config());
        }
        storage::get::<Option<Configuration>>().as_ref().unwrap()
    }

    pub fn exists() -> bool {
        storage::get::<Option<Configuration>>().is_some()
    }

    pub fn save(configuration: Configuration) -> () {
        storage::get_mut::<Option<Configuration>>().replace(configuration);
    }

    pub fn get_default_config() -> Configuration {
        let lambda = Principal::self_authenticating("mltzx-rlg5h-qzcpp-xdp7e-56vnr-cbdjf-e6x5q-gzm2d-2soup-wtk5n-5qe");
        Configuration {
            lambda: lambda,
            token_ttl: Duration::from_secs(60),
            token_refresh_ttl: Duration::from_secs(60),
            whitelisted_phone_numbers: Vec::default(),
            heartbeat: Option::None,
            backup_canister_id: None,
            ii_canister_id: Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap(),
            whitelisted_canisters: None,
            env: None,
            git_branch: None,
            commit_hash: None,
        }
    }
}

pub fn is_anchor_exists(anchor: u64) -> bool { //todo move somewhere
    let accounts = storage::get_mut::<Accounts>();
    accounts.into_iter()
        .map(|l| l.1.anchor)
        .any(|x| x == anchor)
}

pub fn pre_upgrade() {
    canistergeek_ic_rust::logger
    ::log_message("Pre upgrade started".to_string());
    let mut accounts = Vec::new();
    for p in storage::get_mut::<Accounts>().iter() {
        accounts.push(p.1.clone());
    }
    let admin = storage::get_mut::<Option<Principal>>().unwrap();
    let logs = storage::get_mut::<Logs>(); //todo remove somehow
    let logs_new = canistergeek_ic_rust::logger::pre_upgrade_stable_data();
    let monitor_stable_data = canistergeek_ic_rust::monitor::pre_upgrade_stable_data();
    match storage::stable_save((accounts, admin, logs, Some(monitor_stable_data), Some(logs_new), )) { _ => () }; //todo migrate to object
}

pub fn post_upgrade() {
    let (old_accs, admin, _logs, monitor_data, logs_new): (Vec<Account>, Principal, Logs, Option<canistergeek_ic_rust::monitor::PostUpgradeStableData>, Option<canistergeek_ic_rust::logger::PostUpgradeStableData>) = storage::stable_restore().unwrap();
    let mut phone_numbers = HashSet::default();
    storage::get_mut::<Option<Principal>>().replace(admin);
    for u in old_accs {
        let princ = u.clone().principal_id;
        storage::get_mut::<Accounts>().insert(princ.clone(), u.clone());

        storage::get_mut::<PrincipalIndex>().insert(princ.clone(), princ.clone());

        for x in u.clone().access_points.into_iter() {
            storage::get_mut::<PrincipalIndex>().insert(x.principal_id, princ.clone());
        }

        u.phone_number_sha2.map(|x| phone_numbers.insert(x.clone()));
    }
    storage::get_mut::<Option<Principal>>().replace(admin);
    let pn_repo = PhoneNumberRepo {};
    pn_repo.add_all(phone_numbers);
    match monitor_data {
        None => {}
        Some(data) => {
            canistergeek_ic_rust::monitor::post_upgrade_stable_data(data);
        }
    }
    match logs_new {
        None => {}
        Some(log) => {
            canistergeek_ic_rust::logger::post_upgrade_stable_data(log);
        }
    }
    canistergeek_ic_rust::logger
    ::log_message("Post upgrade completed".to_string());
}
