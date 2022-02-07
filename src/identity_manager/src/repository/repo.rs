use std::collections::{BTreeMap, BTreeSet, HashSet};

use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use ic_cdk::storage;

use crate::Configuration;
use crate::repository::encrypt::account_encrypt::{decrypt_phone_number, encrypt};
use crate::repository::encrypted_repo::{EncryptedAccount, EncryptedRepo};

#[derive(Clone, Debug, CandidType, Deserialize, PartialEq)]
pub struct AccessPoint {
    pub pub_key: String,
    pub last_used: String,
    pub make: String,
    pub model: String,
    pub browser: String,
    pub name: String,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Persona {
    pub anchor: Option<u64>,
    pub domain: String,
    pub persona_id: Option<String>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Account {
    pub anchor: u64,
    pub principal_id: String,
    pub name: String,
    pub phone_number: String,
    pub access_points: Vec<AccessPoint>,
    pub personas: Vec<Persona>,
}

#[derive(Clone, Debug, CandidType, Deserialize, PartialOrd, PartialEq, Ord, Eq)]
pub struct Application {
    pub domain: String,
    pub user_limit: u16,
    pub name: String,
}

pub type EncryptedAccounts = BTreeMap<String, EncryptedAccount>;
pub type Applications = BTreeSet<Application>;

type PhoneNumbers = HashSet<blake3::Hash>;

pub struct AccountRepo {}

pub struct AccessPointRepo {}

pub struct PersonaRepo {}

pub struct PhoneNumberRepo {}

pub struct AdminRepo {}

pub struct ConfigurationRepo {}

pub struct ApplicationRepo {}


impl AccountRepo {
    pub fn create_account(account: Account) -> Option<Account> {
        EncryptedRepo::create_account(account)
    }

    pub fn store_account(account: Account) -> Option<Account> {
        EncryptedRepo::store_account(account)
    }

    pub fn get_account() -> Option<Account> {
        EncryptedRepo::get_account()
    }
}

impl AccessPointRepo {
    pub fn get_access_points() -> Option<Vec<AccessPoint>> {
        AccountRepo::get_account()
            .map(|x| x.access_points.clone()) //todo &
    }

    pub fn store_access_points(access_points: Vec<AccessPoint>) -> Option<Account> {
        let mut acc = AccountRepo::get_account()
            .unwrap().clone();
        acc.access_points = access_points;
        AccountRepo::store_account(acc)
    }
}

impl PersonaRepo {
    pub fn get_personas() -> Option<Vec<Persona>> {
        AccountRepo::get_account()
            .map(|x| x.personas.clone()) //todo &
    }

    pub fn store_persona(persona: Persona) -> Option<Account> {
        let acc = AccountRepo::get_account();
        if acc.is_none() { return None; }
        let mut account = acc.unwrap().clone();
        account.personas.push(persona);
        AccountRepo::store_account(account)
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

impl PhoneNumberRepo {
    pub fn is_exist(phone_number_hash: &blake3::Hash) -> bool {
        storage::get::<PhoneNumbers>().contains(phone_number_hash)
    }

    pub fn add(phone_number_hash: blake3::Hash) -> () {
        storage::get_mut::<PhoneNumbers>().insert(phone_number_hash);
    }

    pub fn add_all(phone_number_hashes: HashSet<blake3::Hash>) -> () {
        storage::get_mut::<PhoneNumbers>().extend(phone_number_hashes);
    }
}

impl ConfigurationRepo {
    pub fn get() -> &'static Configuration {
        storage::get::<Option<Configuration>>().as_ref().unwrap()
    }

    pub fn save(configuration: Configuration) -> () {
        storage::get_mut::<Option<Configuration>>().replace(configuration);
    }
}

impl ApplicationRepo {
    pub fn create_application(application: Application) -> Vec<Application> {
        let applications = storage::get_mut::<Applications>();
        applications.insert(application.clone());
        applications.iter()
            .map(|p| p.clone())
            .collect()
    }

    pub fn read_applications() -> Vec<Application> {
        storage::get_mut::<Applications>().iter()
            .map(|p| p.clone())
            .collect()
    }

    pub fn delete_application(name: String) -> bool {
        let app_to_remove = storage::get_mut::<Applications>().iter()
            .find(|a| a.name.eq(&name));
        match app_to_remove {
            None => { false }
            Some(app) => {
                storage::get_mut::<Applications>()
                    .remove(app)
            }
        }
    }

    pub fn is_application_exists(application: &Application) -> bool {
        let applications = storage::get_mut::<Applications>();
        applications.iter()
            .any(|a| a.name.eq(&application.name) || a.domain.eq(&application.domain))
    }
}

pub fn is_anchor_exists(anch: u64) -> bool {
    let anchor = encrypt(anch.to_string());
    let accounts = storage::get_mut::<EncryptedAccounts>();
    accounts.into_iter()
        .map(|l| {
            let c = l.1.clone();
            let mut anchors: Vec<String> = l.1.personas.iter()
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
    let mut accounts = Vec::new();
    for p in storage::get_mut::<EncryptedAccounts>().iter() {
        accounts.push(p.1.clone());
    }
    let admin = storage::get_mut::<Option<Principal>>().unwrap();
    storage::stable_save((accounts, admin));
}

pub fn post_upgrade() {
    let (old_accs, admin): (Vec<EncryptedAccount>, Principal) = storage::stable_restore().unwrap();
    let mut phone_numbers = HashSet::default();
    for u in old_accs {
        storage::get_mut::<EncryptedAccounts>().insert(u.clone().principal_id, u.clone());
        phone_numbers.insert(blake3::keyed_hash(&ConfigurationRepo::get().key, decrypt_phone_number(u).clone().as_bytes()));
    }
    storage::get_mut::<Option<Principal>>().replace(admin);
    PhoneNumberRepo::add_all(phone_numbers);
}
