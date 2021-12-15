use std::collections::BTreeMap;
use crate::service::principle_service::get_principal;
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
    pub principal_id: String,
}


#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Account {
    pub principal_id: String,
    pub name: String,
    pub phone_number: String,
    pub email: String,
    pub devices: Vec<Device>,
    pub personas: Vec<Persona>,
}

type Principals = BTreeMap<String, String>;
type Accounts = BTreeMap<String, Account>;

pub struct AccountRepo {}

pub struct PrincipalIndex {}

pub struct DeviceRepo {}

pub struct PersonaRepo {}

impl AccountRepo {
    pub fn store_account(principal_id: String, account: Account) -> Option<Account> {
        let accounts = storage::get_mut::<Accounts>();
        accounts.insert(principal_id, account.clone()); //todo try_insert
        Option::Some(account)
    }

    pub fn get_account(principal_id: &str) -> Option<&Account> {
        let accounts = storage::get_mut::<Accounts>();
        accounts.get(get_principal(principal_id))
    }
}

impl PrincipalIndex {
    pub fn get_principal(persona_id: &str) -> Option<&String> {
        let principals = storage::get_mut::<Principals>();
        principals.get(persona_id)
    }

    pub fn store_principal(persona_id: &str, root_id: &str) {
        let principals = storage::get_mut::<Principals>();
        principals.insert(persona_id.to_string(), root_id.to_string());
    }
}

impl DeviceRepo {
    pub fn get_devices(principal_id: &str) -> Option<Vec<Device>> {
        AccountRepo::get_account(principal_id)
            .map(|x| x.devices.clone()) //todo &
    }

    pub fn store_devices(principal_id: &str, devices: Vec<Device>) -> Option<Account> {
        let mut acc = AccountRepo::get_account(principal_id)
            .unwrap().clone();
        acc.devices = devices;
        AccountRepo::store_account(acc.principal_id.clone(), acc)
    }
}

impl PersonaRepo {
    pub fn get_personas(principal_id: &str) -> Option<Vec<Persona>> {
        AccountRepo::get_account(principal_id)
            .map(|x| x.personas.clone()) //todo &
    }

    pub fn get_persona(persona_id: &str) -> Option<Persona> {
        AccountRepo::get_account( get_principal(persona_id))
            .map(|x| x.personas.clone())
            .unwrap()
            .into_iter()
            .find(|x| x.principal_id.eq(&String::from(persona_id.clone())))
    }

    pub fn store_personas(principal_id: &str, personas: Vec<Persona>) -> Option<Account> {
        let mut acc = AccountRepo::get_account(principal_id)
            .unwrap().clone();
        acc.personas = personas;
        AccountRepo::store_account(acc.principal_id.clone(), acc)
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
        storage::get_mut::<Accounts>().insert(u.clone().principal_id, u.clone());
        for persona in u.personas.clone() {
            storage::get_mut::<Principals>().insert(persona.principal_id.clone(), u.clone().principal_id);
        }
    }
}
