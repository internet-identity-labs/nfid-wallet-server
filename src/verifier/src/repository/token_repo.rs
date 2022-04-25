use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;

use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::trap;

use crate::{Credential, HttpResponse};
use crate::repository::credential_repo::store_credential;

#[derive(Clone, Debug, Deserialize, CandidType, Hash, Eq, PartialEq)]
pub struct Token {
    pub client_principal: String,
    pub domain: String,
}

pub type TokenKey = [u8; 32];

thread_local! {
    static TOKEN_STORAGE: RefCell<HashMap<TokenKey, Token>> = RefCell::new(HashMap::new());

}

pub fn insert_certificate(cert: Token, token: TokenKey) -> TokenKey {
    TOKEN_STORAGE.with(|mut storage| {
        let mut st = storage.borrow_mut();
        st.insert(token.clone(), cert.clone());
        token
    })
}

pub fn get_domain(token: TokenKey) -> String {
    let t = TOKEN_STORAGE.with(|storage| {
        let mut st = storage.borrow_mut();
        match st.get(&token) {
            Some(cert) => {
                cert.domain.clone()
            }
            None => { trap("Certificate does not contain domain") }
        }
    });
    t
}

pub fn resolve_certificate(a: Option<String>, token: TokenKey) -> Option<Credential> {
    let tt = TOKEN_STORAGE.with(|storage| {
        let mut st = storage.borrow_mut();
        match st.remove(&token) {
            Some(token) => {
                return token;
            }
            None => { trap("No token with such key") }
        }
    });
    let cert = Credential {
        client_principal: tt.client_principal,
        domain: tt.domain,
        phone_number_sha2: a,
    };
    store_credential(cert)
}

