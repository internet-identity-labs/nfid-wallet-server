use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::{Hash};
use std::time::Duration;
use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use ic_cdk::storage;
use crate::{ic_service, Log, LogLevel, LogRepo};
use crate::logger::logger::Logs;
use crate::repository::account_repo::{Account, Accounts, PrincipalIndex};
use crate::repository::application_repo::Application;
use crate::repository::encrypt::account_encrypt::encrypt;

use crate::repository::phone_number_repo::{PhoneNumberRepo, PhoneNumberRepoTrait};


#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub lambda: Principal,
    pub token_ttl: Duration,
    pub token_refresh_ttl: Duration,
    pub key: [u8; 32],
    pub whitelisted: Vec<String>,
    pub heartbeat: u32,
    pub backup_canister_id: String,
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
        storage::get::<Option<Configuration>>().as_ref().unwrap()
    }

    pub fn save(configuration: Configuration) -> () {
        storage::get_mut::<Option<Configuration>>().replace(configuration);
    }
}


pub fn is_anchor_exists(anchor: u64) -> bool { //todo move somewhere
    let accounts = storage::get_mut::<Accounts>();
    accounts.into_iter()
        .map(|l| {
            let c = l.1.clone();
            let mut anchors:Vec<u64> = l.1.personas.iter()
                .map(|k| k.anchor.clone())
                .filter(|l| l.is_some())
                .map(|l| l.unwrap())
                .collect();
            anchors.push(c.anchor);
            anchors
        })
        .flat_map(|x| x.into_iter())
        .any(|x| x == anchor)
}

pub fn pre_upgrade() {
    LogRepo::save(Log {
        level: LogLevel::INFO,
        log: "Pre upgrade started".to_string(),
        timestamp: ic_service::get_time(),
    });
    let mut accounts = Vec::new();
    for p in storage::get_mut::<Accounts>().iter() {
        accounts.push(p.1.clone());
    }
    let admin = storage::get_mut::<Option<Principal>>().unwrap();
    let logs = storage::get_mut::<Logs>();
    match storage::stable_save((accounts, admin, logs)) { _ => () };
}

pub fn post_upgrade() {
    let (old_accs, admin, logs): (Vec<Account>, Principal, Logs) = storage::stable_restore().unwrap();
    let mut phone_numbers = HashSet::default();
    storage::get_mut::<Option<Principal>>().replace(admin);
    for u in old_accs {
        let princ = u.clone().principal_id;
        storage::get_mut::<Accounts>().insert(princ.clone(), u.clone());

        storage::get_mut::<PrincipalIndex>().insert(princ.clone(), princ.clone());

        for x in u.clone().access_points.into_iter() {
            storage::get_mut::<PrincipalIndex>().insert(x.principal_id, princ.clone());
        }

        u.phone_number.map(|x| phone_numbers.insert(x.clone()));
    }
    storage::get_mut::<Option<Principal>>().replace(admin);
    let pn_repo = PhoneNumberRepo {}; //TODO test container
    pn_repo.add_all(phone_numbers);

    for log in logs {
        storage::get_mut::<Logs>().push(log);
    }

    LogRepo::save(Log {
        level: LogLevel::INFO,
        log: "Post upgrade completed".to_string(),
        timestamp: ic_service::get_time(),
    });
}
