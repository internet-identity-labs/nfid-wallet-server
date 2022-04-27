use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use ic_cdk::export::candid::{CandidType, Deserialize};

use ic_cdk::storage;
use ic_cdk::export::Principal;
use crate::AdminRepo;

#[derive(Clone, Debug, Deserialize, CandidType, Hash, Eq, PartialEq)]
pub struct Credential {
    pub client_principal: String,
    pub domain: String,
    pub phone_number_sha2: Option<String>,
    pub created_date: u64,
}

thread_local! {
        static CERTIFICATE_STORAGE: RefCell<HashMap<String, Credential >> = RefCell::new(HashMap::new());
}

pub fn store_credential(certificate: Credential) -> Option<Credential> {
    CERTIFICATE_STORAGE.with(|stable_storage| {
        let mut cert_st = stable_storage.borrow_mut();
        cert_st.insert(certificate.client_principal.clone(), certificate.clone());
        Some(certificate)
    })
}

pub fn get_credential(who: String) -> Option<Credential> {
    CERTIFICATE_STORAGE.with(|storage| {
        let mut st = storage.borrow_mut();
        match st.get(&who) {
            None => { None }
            Some(t) => { Some(t.to_owned()) }
        }
    })
}


pub fn pre_upgrade() {
    CERTIFICATE_STORAGE.with(|storage| {
        let mut st = storage.borrow_mut();
        let mut certs = HashSet::new();

        for cc in st.iter() {
            certs.insert(cc.1.to_owned());
        }
        storage::stable_save((certs, AdminRepo::get()));
    });
}

pub fn post_upgrade() {
    let (certs, admin): (HashSet<Credential>, Principal) = storage::stable_restore().unwrap();
    AdminRepo::save(admin);
    CERTIFICATE_STORAGE.with(|storage| {
        let mut st = storage.borrow_mut();
        for p in certs.iter() {
            let cert = p.to_owned();
            st.insert(cert.client_principal.clone(), cert);
        }
    });
}
