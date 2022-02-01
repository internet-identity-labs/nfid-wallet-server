use std::collections::{BTreeMap, HashSet};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use ic_cdk::{storage};
use crate::Configuration;

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

type Accounts = BTreeMap<String, Account>;
type PhoneNumbers = HashSet<blake3::Hash>;

pub struct AccountRepo {}

pub struct AccessPointRepo {}

pub struct PersonaRepo {}

pub struct PhoneNumberRepo {}

pub struct AdminRepo {}

pub struct ConfigurationRepo {}


impl AccountRepo {
    pub fn create_account(account: Account) -> Option<Account> {
        let accounts = storage::get_mut::<Accounts>();
        if is_anchor_exists(account.anchor) {
            None
        } else {
            accounts.insert(account.principal_id.clone(), account.clone());
            Some(account)
        }
    }

    pub fn store_account(account: Account) -> Option<Account> {
        let accounts = storage::get_mut::<Accounts>();
        accounts.insert(account.principal_id.clone(), account.clone()); //todo try_insert
        Some(account)
    }

    pub fn get_account() -> Option<&'static Account> {
        let princ = &ic_cdk::api::caller().to_text();
        let accounts = storage::get_mut::<Accounts>();
        accounts.get(princ)
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

    pub fn store_personas(personas: Vec<Persona>) -> Option<Account> {
        let mut acc = AccountRepo::get_account()
            .unwrap().clone();
        acc.personas = personas;
        AccountRepo::store_account(acc)
    }

    pub fn store_persona(persona: Persona) -> Option<Account> {
        let mut acc = AccountRepo::get_account();
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

pub fn is_anchor_exists(anchor: u64) -> bool {
    let accounts = storage::get_mut::<Accounts>();
    accounts.into_iter()
        .map(|l| {
            let mut anchors: Vec<u64> = l.1.personas.iter()
                .map(|k| k.anchor)
                .filter(|l| l.is_some())
                .map(|l| l.unwrap())
                .collect();
            anchors.push(l.1.anchor);
            anchors
        })
        .flat_map(|x| x.into_iter())
        .any(|x| x == anchor)
}
