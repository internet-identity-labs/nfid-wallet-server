use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;

use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::trap;

use crate::{ConfigurationRepo, Credential, HttpResponse};
use crate::repository::credential_repo::store_credential;

#[derive(Clone, Debug, Deserialize, CandidType, Hash, Eq, PartialEq)]
pub struct Token {
    pub client_principal: String,
    pub domain: String,
    pub created_date: u64,
}

pub type TokenKey = [u8; 32];

thread_local! {
    static TOKEN_STORAGE: RefCell<HashMap<TokenKey, Token>> = RefCell::new(HashMap::new());

}

pub fn generate_token(client_principal: String, domain: String, token: TokenKey) -> TokenKey {
    let cert = Token {
        client_principal,
        domain,
        created_date: ic_cdk::api::time(),
    };
    TOKEN_STORAGE.with(|mut storage| {
        let mut st = storage.borrow_mut();
        st.insert(token.clone(), cert.clone());
        token
    })
}

pub fn resolve_token(token: TokenKey) -> String {
    TOKEN_STORAGE.with(|storage| {
        let mut st = storage.borrow_mut();
        let now = ic_cdk::api::time();
        // st.retain(|_, l| (l.created_date + ConfigurationRepo::get().token_ttl) > now);
        match st.get(&token) {
            Some(cert) => {
                cert.domain.clone()
            }
            None => { trap("Certificate does not contain domain") }
        }
    })
}

pub fn resolve_certificate(a: Option<String>, token: TokenKey) -> Option<Credential> {
    let token = TOKEN_STORAGE.with(|storage| {
        let mut st = storage.borrow_mut();
        match st.remove(&token) {
            Some(token) => {
                return token;
            }
            None => { trap("No token with such key") }
        }
    });
    let cert = Credential {
        client_principal: token.client_principal,
        domain: token.domain,
        phone_number_sha2: a,
        created_date: ic_cdk::api::time(),
    };
    store_credential(cert)
}

