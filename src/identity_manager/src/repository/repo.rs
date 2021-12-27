use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::storage;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Device {
    pub pub_key_hash: String,
    pub last_used: String,
    pub make: String,
    pub model: String,
    pub browser: String,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Persona {
    pub name: String,
    pub is_root: bool,
    pub is_seed_phrase_copied: bool,
    pub is_ii_anchor: bool,
    pub anchor: String,
    pub principal_id_hash: u64,
    pub principal_id: String,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Account {
    pub principal_id_hash: u64,
    pub principal_id: String,
    pub name: String,
    pub phone_number: String,
    pub email: String,
    pub devices: Vec<Device>,
    pub personas: Vec<Persona>,
}

type Principals = BTreeMap<u64, u64>;
type Accounts = BTreeMap<u64, Account>;

pub struct AccountRepo {}

pub struct PrincipalIndex {}

pub struct DeviceRepo {}

pub struct PersonaRepo {}

impl AccountRepo {
    pub fn store_account(account: Account) -> Option<Account> {
        let accounts = storage::get_mut::<Accounts>();
        accounts.insert(account.principal_id_hash, account.clone()); //todo try_insert
        Option::Some(account)
    }

    pub fn get_account() -> Option<&'static Account> {
        let princ = calculate_hash(&ic_cdk::api::caller().to_text());
        let accounts = storage::get_mut::<Accounts>();
        accounts.get(&get_principal(princ))
    }
}

impl PrincipalIndex {
    pub fn get_principal(persona_id: u64) -> Option<&'static u64> {
        let principals = storage::get_mut::<Principals>();
        principals.get(&persona_id)
    }

    pub fn store_principal(persona_id: u64) {
        let princ = calculate_hash(&ic_cdk::api::caller().to_text());
        let root_id = get_principal(princ);
        let principals = storage::get_mut::<Principals>();
        principals.insert(persona_id, root_id);
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

    pub fn get_persona(persona_id: String) -> Option<Persona> {
        AccountRepo::get_account()
            .map(|x| x.personas.clone())
            .unwrap()
            .into_iter()
            .find(|x| x.principal_id.eq(&persona_id))
    }

    pub fn store_personas(personas: Vec<Persona>) -> Option<Account> {
        let mut acc = AccountRepo::get_account()
            .unwrap().clone();
        acc.personas = personas;
        AccountRepo::store_account(acc)
    }
}

pub fn pre_upgrade() {
    let mut accounts = Vec::new();
    for p in storage::get_mut::<Accounts>().iter() {
        accounts.push(p.1.clone());
    }
    storage::stable_save((accounts, )).unwrap();
}

pub fn post_upgrade() {
    let (old_accs, ): (Vec<Account>, ) = storage::stable_restore().unwrap();
    for u in old_accs {
        storage::get_mut::<Accounts>().insert(u.clone().principal_id_hash, u.clone());
        for persona in u.personas.clone() {
            storage::get_mut::<Principals>().insert(persona.principal_id_hash.clone(), u.clone().principal_id_hash);
        }
    }
}

pub fn calculate_hash<T: Hash + ?Sized>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

pub fn get_principal(persona_id: u64) -> u64 {
    *match PrincipalIndex::get_principal(persona_id) {
        Some(principal_id) => {
            principal_id
        }
        None => {
            &persona_id
        }
    }
}
