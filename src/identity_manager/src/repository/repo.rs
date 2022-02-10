use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::time::Duration;
use ic_cdk::api::time;

use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use ic_cdk::storage;


use crate::repository::encrypt::account_encrypt::{decrypt_phone_number, encrypt};
use crate::repository::encrypt::encrypted_repo::{EncryptedAccount};
use crate::repository::phone_number_repo::{PhoneNumberRepo, PhoneNumberRepoTrait};

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

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub lambda: Principal,
    pub token_ttl: Duration,
    pub token_refresh_ttl: Duration,
    pub key: [u8; 32],
    pub whitelisted: Vec<String>,
}

pub type EncryptedAccounts = BTreeMap<String, EncryptedAccount>;
//todo rethink visibility
pub type Applications = BTreeSet<Application>;
pub type Tokens = HashMap<blake3::Hash, (blake3::Hash, u64)>;
pub type PhoneNumbers = HashSet<blake3::Hash>;

pub struct TokenRepo {}

pub struct AdminRepo {}

pub struct ConfigurationRepo {}


impl AdminRepo {
    pub fn get() -> Principal {
        storage::get_mut::<Option<Principal>>().unwrap()
    }

    pub fn save(principal: Principal) -> () {
        storage::get_mut::<Option<Principal>>().replace(principal);
    }
}

impl TokenRepo {
    pub fn add(phone_number: blake3::Hash, token: blake3::Hash) -> () {
        let time_window: u64 = time() - ConfigurationRepo::get().token_ttl.as_nanos() as u64;
        storage::get_mut::<Tokens>().retain(|_, (_, t)| *t > time_window);
        storage::get_mut::<Tokens>().insert(phone_number, (token, time()));
    }

    pub fn get(phone_number: &blake3::Hash, duration: Duration) -> Option<&blake3::Hash> {
        let time_window: u64 = time() - duration.as_nanos() as u64;
        storage::get::<Tokens>()
            .get(phone_number)
            .filter(|(_, t)| *t > time_window)
            .map(|(v, _)| v)
    }
}

impl ConfigurationRepo { //todo fix Principle not implement default!
    pub fn get() -> &'static Configuration {
        storage::get::<Option<Configuration>>().as_ref().unwrap()
    }

    pub fn save(configuration: Configuration) -> () {
        storage::get_mut::<Option<Configuration>>().replace(configuration);
    }
}


pub fn is_anchor_exists(anch: u64) -> bool { //todo move somewhere
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
    match storage::stable_save((accounts, admin)) { _ => () };
}

pub fn post_upgrade() {
    let (old_accs, admin): (Vec<EncryptedAccount>, Principal) = storage::stable_restore().unwrap();
    let mut phone_numbers = HashSet::default();
    for u in old_accs {
        storage::get_mut::<EncryptedAccounts>().insert(u.clone().principal_id, u.clone());
        phone_numbers.insert(blake3::keyed_hash(&ConfigurationRepo::get().key, decrypt_phone_number(u).clone().as_bytes()));
    }
    storage::get_mut::<Option<Principal>>().replace(admin);
    let pn_repo = PhoneNumberRepo {}; //TODO test container
    pn_repo.add_all(phone_numbers);
}
