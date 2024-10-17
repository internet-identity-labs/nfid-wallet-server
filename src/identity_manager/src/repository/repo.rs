use std::collections::{BTreeSet, HashSet};
use std::hash::Hash;
use std::time::Duration;
use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use ic_cdk::storage;
use crate::ic_service;
use crate::logger::logger::Logs;
use crate::repository::account_repo::{Account, Accounts, PrincipalIndex};
use crate::repository::application_repo::Application;
use crate::service::device_index_service;
use serde::Serialize;
use crate::http::requests::{DeviceType, WalletVariant};
use crate::repository::access_point_repo::AccessPoint;
use crate::repository::persona_repo::Persona;


#[derive(Debug, Deserialize, CandidType, Clone)]
pub struct Configuration {
    pub lambda_url: String,
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
    pub operator: Principal,
}

//todo rethink visibility
pub type Applications = BTreeSet<Application>;

pub struct AdminRepo {}

pub struct ControllersRepo {}

pub struct ConfigurationRepo {}

#[derive(Clone, Debug, CandidType, Deserialize, Default, PartialEq, Eq, Copy, Hash, Serialize)]
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
        let lambda = Principal::from_text("ritih-icnvs-i7b67-sc2vs-nwo2e-bvpe5-viznv-uqluj-xzcvs-6iqsp-fqe").unwrap();
        Configuration {
            lambda_url: "https://d8m9ttp390ku4.cloudfront.net/dev".to_string(),
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
            operator: lambda,
        }
    }
}

pub fn is_anchor_exists(anchor: u64, wallet: WalletVariant) -> bool {
    let accounts = storage::get_mut::<Accounts>();
    accounts.into_iter()
        .map(|l| l.1)
        .any(|x| x.anchor == anchor && x.wallet.eq(&wallet))
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct AccountMemoryModel {
    pub anchor: u64,
    pub principal_id: String,
    pub name: Option<String>,
    pub personas: Vec<Persona>,
    pub phone_number: Option<String>,
    pub phone_number_sha2: Option<String>,
    pub access_points: HashSet<AccessPointMemoryModel>,
    pub base_fields: BasicEntity,
    pub wallet: Option<WalletVariant>,
    pub is2fa_enabled: Option<bool>,
    pub email: Option<String>,
}


#[derive(Clone, Debug, CandidType, Deserialize,PartialEq ,Eq, Serialize, Hash)]
pub struct AccessPointMemoryModel {
    pub principal_id: String,
    pub credential_id: Option<String>,
    pub icon: Option<String>,
    pub device: Option<String>,
    pub browser: Option<String>,
    pub last_used: Option<u64>,
    pub device_type: Option<DeviceType>,
    pub base_fields: BasicEntity,
}


pub fn pre_upgrade() {
    let mut accounts = Vec::new();
    for p in storage::get_mut::<Accounts>().iter() {
        accounts.push(
            AccountMemoryModel {
                anchor: p.1.anchor.clone(),
                principal_id: p.1.principal_id.to_string(),
                name: p.1.name.clone(),
                personas: p.1.personas.clone(),
                phone_number: p.1.phone_number.clone(),
                phone_number_sha2: p.1.phone_number_sha2.clone(),
                access_points: p.1.access_points.clone()
                    .into_iter().map(|ap|access_point_to_memory_model(ap))
                    .collect(),
                base_fields: p.1.base_fields.clone(),
                wallet: Some(p.1.wallet.clone()),
                is2fa_enabled: Some(p.1.is2fa_enabled),
                email: p.1.email.clone(),
        })
    }
    let admin = storage::get_mut::<Option<Principal>>().unwrap();
    let logs = storage::get_mut::<Logs>(); //todo remove somehow
    let applications = storage::get_mut::<Applications>();
    let configuration = ConfigurationRepo::get().clone();
    match storage::stable_save((accounts, admin, logs, Some(applications), Some(configuration))) { _ => () }; //todo migrate to object
}

pub fn post_upgrade() {
    let (old_accs, admin, _logs, applications, configuration_maybe): (Vec<AccountMemoryModel>, Principal, Logs, Option<Applications>, Option<Configuration>) = storage::stable_restore().unwrap();
    let configuration = configuration_maybe.unwrap_or(ConfigurationRepo::get_default_config());
    ConfigurationRepo::save(configuration);
    storage::get_mut::<Option<Principal>>().replace(admin);
    for u in old_accs {
        let princ = u.principal_id.clone();
        storage::get_mut::<Accounts>().insert(princ.clone(), Account {
            anchor: u.anchor,
            principal_id: u.principal_id.to_string(),
            name: u.name,
            phone_number: u.phone_number,
            phone_number_sha2: u.phone_number_sha2,
            personas: u.personas,
            access_points: u.access_points.clone()
                .into_iter()
                .map(|ap| access_point_mm_to_ap(ap))
                .collect(),
            base_fields: u.base_fields,
            wallet: match u.wallet {
                None => { WalletVariant::InternetIdentity }
                Some(x) => { x }
            },
            is2fa_enabled: match u.is2fa_enabled {
                None => { false }
                Some(x) => { x }
            },
            email: u.email,
        });
        for x in u.access_points.into_iter() {
            storage::get_mut::<PrincipalIndex>().insert(x.principal_id, princ.clone());
        }
        storage::get_mut::<Option<Principal>>().replace(admin);
    }
    match applications {
        None => {}
        Some(applications) => {
            let apps = storage::get_mut::<Applications>();
            applications.iter().for_each(|a| {
                apps.insert(a.to_owned());
            })
        }
    }
}

fn access_point_to_memory_model(ap: AccessPoint) -> AccessPointMemoryModel {
    return AccessPointMemoryModel {
        principal_id: ap.principal_id,
        credential_id: ap.credential_id,
        icon: ap.icon,
        device: ap.device,
        browser: ap.browser,
        last_used: ap.last_used,
        device_type: Some(ap.device_type),
        base_fields: ap.base_fields,
    };
}

fn access_point_mm_to_ap(ap: AccessPointMemoryModel) -> AccessPoint {
    return AccessPoint {
        principal_id: ap.principal_id,
        credential_id: ap.credential_id,
        icon: ap.icon,
        device: ap.device,
        browser: ap.browser,
        last_used: ap.last_used,
        device_type: match ap.device_type {
            None => { DeviceType::Unknown }
            Some(x) => { x }
        },
        base_fields: ap.base_fields,
    };
}