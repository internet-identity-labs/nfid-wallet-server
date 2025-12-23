use crate::http::requests::{DeviceType, WalletVariant};
use crate::ic_service;
use crate::logger::logger::Logs;
use crate::repository::access_point_repo::AccessPoint;
use crate::repository::account_repo::{Account, ACCOUNTS, PRINCIPAL_INDEX};
use crate::repository::application_repo::Application;
use crate::repository::persona_repo::Persona;
use crate::structure::ttl_hashmap::TtlHashMap;
use candid::{CandidType, Deserialize, Principal};
use ic_cdk::storage;
use serde::Serialize;
use std::cell::RefCell;
use std::collections::{BTreeSet, HashSet};
use std::hash::Hash;
use std::time::Duration;

const fn secs_to_nanos(secs: u64) -> u64 {
    secs * 1_000_000_000
}
pub const MINUTE_NS: u64 = secs_to_nanos(60);
const TEMP_KEY_EXPIRATION_NS: u64 = 10 * MINUTE_NS;
const CAPTCHA_KEY_EXPIRATION_NS: u64 = 5 * MINUTE_NS;

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
    pub account_creation_paused: bool,
    pub max_free_captcha_per_minute: u16,
    pub test_captcha: bool,
}

//todo rethink visibility
pub type Applications = BTreeSet<Application>;

thread_local! {
  pub static APPLICATIONS: RefCell<BTreeSet<Application>> = const { RefCell::new(BTreeSet::new()) };
  pub static TEMP_KEYS: RefCell<TtlHashMap<String, u64>> = RefCell::new(TtlHashMap::new(TEMP_KEY_EXPIRATION_NS));
  pub static CAPTCHA_CAHLLENGES: RefCell<TtlHashMap<String, Option<String>>> = RefCell::new(TtlHashMap::new(CAPTCHA_KEY_EXPIRATION_NS));
    pub static ADMINS: RefCell<HashSet<Principal>> = RefCell::new(HashSet::new());
    pub static CONTROLLERS: RefCell<HashSet<Principal>> = RefCell::new(HashSet::new());
    pub static CONFIGURATION: RefCell<Configuration> = RefCell::new(ConfigurationRepo::get_default_config());

}

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
        BasicEntity {
            created_date: ic_service::get_time(),
            modified_date: ic_service::get_time(),
        }
    }
}

impl AdminRepo {
    pub fn get() -> Principal {
        ADMINS.with(|admins| {
            *admins
                .borrow()
                .iter()
                .next()
                .expect("Failed to retrieve an admin. The admin list is empty.")
        })
    }

    pub fn save(principal: Principal) {
        ADMINS.with(|admins| {
            admins.borrow_mut().insert(principal);
        });
    }
}

impl ControllersRepo {
    pub fn get() -> Vec<Principal> {
        CONTROLLERS.with(|controllers| controllers.borrow().iter().copied().collect())
    }

    pub fn save(principals: Vec<Principal>) {
        CONTROLLERS.with(|controllers| {
            for p in principals {
                controllers.borrow_mut().insert(p);
            }
        });
    }

    pub fn contains(principal: &Principal) -> bool {
        CONTROLLERS.with(|controllers| controllers.borrow().contains(principal))
    }
}

impl ConfigurationRepo {
    //todo fix Principle not implement default!
    pub fn get() -> Configuration {
        CONFIGURATION.with(|config| config.borrow().clone())
    }

    pub fn save(configuration: Configuration) {
        CONFIGURATION.with(|config| {
            config.replace(configuration);
        });
    }

    pub fn get_default_config() -> Configuration {
        let lambda =
            Principal::from_text("ritih-icnvs-i7b67-sc2vs-nwo2e-bvpe5-viznv-uqluj-xzcvs-6iqsp-fqe")
                .expect("Failed to parse the lambda principal string.");
        Configuration {
            lambda_url: "https://d8m9ttp390ku4.cloudfront.net/dev".to_string(),
            lambda,
            token_ttl: Duration::from_secs(60),
            token_refresh_ttl: Duration::from_secs(60),
            whitelisted_phone_numbers: Vec::default(),
            heartbeat: Option::None,
            backup_canister_id: None,
            ii_canister_id: Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai")
                .expect("Failed to parse the ii_canister_id string."),
            whitelisted_canisters: None,
            env: None,
            git_branch: None,
            commit_hash: None,
            operator: lambda,
            account_creation_paused: false,
            max_free_captcha_per_minute: 10,
            test_captcha: false,
        }
    }
}

pub fn is_anchor_exists(anchor: u64, wallet: WalletVariant) -> bool {
    ACCOUNTS.with(|accounts| {
        accounts
            .borrow()
            .iter()
            .any(|x| x.1.anchor == anchor && x.1.wallet.eq(&wallet))
    })
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

#[derive(Clone, Debug, CandidType, Deserialize, PartialEq, Eq, Serialize, Hash)]
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
    ACCOUNTS.with(|accounts_struct| {
        for p in accounts_struct.borrow().iter() {
            accounts.push(AccountMemoryModel {
                anchor: p.1.anchor,
                principal_id: p.1.principal_id.to_string(),
                name: p.1.name.clone(),
                personas: p.1.personas.clone(),
                phone_number: p.1.phone_number.clone(),
                phone_number_sha2: p.1.phone_number_sha2.clone(),
                access_points: p
                    .1
                    .access_points
                    .clone()
                    .into_iter()
                    .map(access_point_to_memory_model)
                    .collect(),
                base_fields: p.1.base_fields,
                wallet: Some(p.1.wallet),
                is2fa_enabled: Some(p.1.is2fa_enabled),
                email: p.1.email.clone(),
            })
        }
    });
    let admin = AdminRepo::get();
    let logs: Vec<Logs> = Vec::default(); //todo remove somehow
    let applications = APPLICATIONS.with(|apps| apps.borrow().clone());
    let configuration = CONFIGURATION.with(|config| config.borrow().clone());
    match storage::stable_save((
        accounts,
        admin,
        logs,
        Some(applications),
        Some(configuration),
    )) {
        _ => (),
    }; //todo migrate to object
}

pub fn post_upgrade() {
    let (old_accs, admin, _logs, applications, configuration_maybe): (
        Vec<AccountMemoryModel>,
        Principal,
        Logs,
        Option<Applications>,
        Option<Configuration>,
    ) = storage::stable_restore()
        .expect("Stable restore exited unexpectedly: unable to restore data from stable memory.");
    CONFIGURATION.with(|config| {
        let configuration = configuration_maybe.unwrap_or(ConfigurationRepo::get_default_config());
        config.replace(configuration);
    });
    ADMINS.with(|admins| {
        admins.borrow_mut().insert(admin);
    });
    for u in old_accs {
        let princ = u.principal_id.clone();

        PRINCIPAL_INDEX.with(|index| {
            for x in u.access_points.clone().into_iter() {
                index
                    .borrow_mut()
                    .insert(x.principal_id.clone(), princ.clone());
            }
        });
        ACCOUNTS.with(|accounts| {
            accounts.borrow_mut().insert(
                princ.clone(),
                Account {
                    anchor: u.anchor,
                    principal_id: u.principal_id.to_string(),
                    name: u.name,
                    phone_number: None,
                    phone_number_sha2: None,
                    personas: u.personas,
                    access_points: u
                        .access_points
                        .clone()
                        .into_iter()
                        .map(access_point_mm_to_ap)
                        .collect(),
                    base_fields: u.base_fields,
                    wallet: match u.wallet {
                        None => WalletVariant::InternetIdentity,
                        Some(x) => x,
                    },
                    is2fa_enabled: match u.is2fa_enabled {
                        None => false,
                        Some(x) => x,
                    },
                    email: u.email,
                },
            );
        });
    }
    match applications {
        None => {}
        Some(applications) => APPLICATIONS.with(|apps| {
            applications.iter().for_each(|a| {
                apps.borrow_mut().insert(a.to_owned());
            })
        }),
    }
}

fn access_point_to_memory_model(ap: AccessPoint) -> AccessPointMemoryModel {
    AccessPointMemoryModel {
        principal_id: ap.principal_id,
        credential_id: ap.credential_id,
        icon: ap.icon,
        device: ap.device,
        browser: ap.browser,
        last_used: ap.last_used,
        device_type: Some(ap.device_type),
        base_fields: ap.base_fields,
    }
}

fn access_point_mm_to_ap(ap: AccessPointMemoryModel) -> AccessPoint {
    AccessPoint {
        principal_id: ap.principal_id,
        credential_id: ap.credential_id,
        icon: ap.icon,
        device: ap.device,
        browser: ap.browser,
        last_used: ap.last_used,
        device_type: match ap.device_type {
            None => DeviceType::Unknown,
            Some(x) => x,
        },
        base_fields: ap.base_fields,
    }
}
