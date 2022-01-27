use std::collections::{BTreeMap, HashSet};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::str::FromStr;

use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use ic_cdk::storage;
use crate::Configuration;

#[derive(Clone, Debug, CandidType, Deserialize, PartialEq)]
pub struct Device {
    pub pub_key_hash: String,
    pub last_used: String,
    pub make: String,
    pub model: String,
    pub browser: String,
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
    pub devices: Vec<Device>,
    pub personas: Vec<Persona>,
}

type Accounts = BTreeMap<String, Account>;
type PhoneNumbers = HashSet<blake3::Hash>;

pub struct AccountRepo {}

pub struct DeviceRepo {}

pub struct PersonaRepo {}

pub struct PhoneNumberRepo {}

pub struct AdminRepo {}

pub struct ConfigurationRepo {}

impl AccountRepo {
    pub fn store_account(account: Account) -> Option<Account> {
        let accounts = storage::get_mut::<Accounts>();
        accounts.insert( account.principal_id.clone(), account.clone()); //todo try_insert
        Option::Some(account)
    }

    pub fn get_account() -> Option<&'static Account> {
        let princ = &ic_cdk::api::caller().to_text();
        let accounts = storage::get_mut::<Accounts>();
        accounts.get(princ)
    }
}

impl DeviceRepo {
    pub fn get_devices() -> Option<Vec<Device>> {
        AccountRepo::get_account()
            .map(|x| x.devices.clone()) //todo &
    }

    pub fn store_devices(devices: Vec<Device>) -> Option<Account> {
        let mut acc = AccountRepo::get_account()
            .unwrap().clone();
        acc.devices = devices;
        AccountRepo::store_account(acc)
    }
}

impl PersonaRepo {
    pub fn get_personas() -> Option<Vec<Persona>> {
        AccountRepo::get_account()
            .map(|x| x.personas.clone()) //todo &
    }

    pub fn store_personas(personas: Vec<Persona>) -> Option<Account> {
        let mut acc = AccountRepo::get_account()
            .unwrap().clone();
        acc.personas = personas;
        AccountRepo::store_account(acc)
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
    pub fn get() -> Configuration {
        storage::get::<Option<Configuration>>().unwrap()
    }

    pub fn save(configuration: Configuration) -> () {
        storage::get_mut::<Option<Configuration>>().replace(configuration);
    }
}

pub fn pre_upgrade() {
    let mut accounts = Vec::new();
    for p in storage::get_mut::<Accounts>().iter() {
        accounts.push(p.1.clone());
    }

    let admin = storage::get_mut::<Option<Principal>>().unwrap();
    storage::stable_save((accounts, admin));
}

pub fn post_upgrade() {
    let (old_accs, admin): (Vec<Account>, Principal) = storage::stable_restore().unwrap();
    let mut phone_numbers = HashSet::default();
    for u in old_accs {
        storage::get_mut::<Accounts>().insert(u.clone().principal_id, u.clone());
        phone_numbers.insert(blake3::keyed_hash(&ConfigurationRepo::get().key, u.clone().phone_number.as_bytes()));
    }
    storage::get_mut::<Option<Principal>>().replace(admin);
    PhoneNumberRepo::add_all(phone_numbers);
}

pub fn calculate_hash<T: Hash + ?Sized>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
