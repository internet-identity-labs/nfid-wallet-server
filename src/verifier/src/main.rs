use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::convert::TryInto;
use std::io::Cursor;
use std::option::Option;
use std::ptr::null;
use std::str;
use std::time::Duration;

use ic_cdk::{call, print, storage, trap};
use ic_cdk::export::candid::{CandidType, Deserialize};

use ic_cdk_macros::{query, update};
use ic_cdk_macros::*;

use canister_api_macros::{admin, log_error, replicate_account};

use crate::credential_repo::Credential;
use crate::http::request::ConfigurationRequest;
use crate::http::responses::HttpResponse;
use crate::repository::configuration_repo::{AdminRepo, Configuration, ConfigurationRepo};
use crate::repository::credential_repo;
use crate::repository::token_repo::{resolve_certificate, Token, TokenKey};
use crate::service::credential_service;
use crate::service::ic_service::get_caller;
use crate::service::im_service::verify_phone_number_existence;

mod repository;
mod service;
mod http;


#[init]
async fn init() -> () {
    AdminRepo::save(get_caller());
}


#[update]
#[admin]
async fn configure(request: ConfigurationRequest) -> () {
    let configuration = Configuration {
        identity_manager_canister_id: request.identity_manager,
        whitelisted_canisters: None,
        token_ttl:  Duration::from_secs( request.token_ttl.unwrap_or(60))
    };
    ConfigurationRepo::save(configuration);
}

#[update]
async fn generate_pn_token(domain: String) -> TokenKey {
    credential_service::generate_pn_token(domain).await
}

#[query]
async fn is_phone_number_approved(who: String) -> HttpResponse<bool> {
    credential_service::is_phone_number_approved(who)
}

#[update]
async fn resolve_token(token_key: TokenKey) -> Option<Credential> {
    credential_service::resolve_token(token_key).await
}


fn main() {}


#[pre_upgrade]
async fn pre_upgrade() {
    credential_repo::pre_upgrade()
}

#[post_upgrade]
fn post_upgrade() {
    credential_repo::post_upgrade();
}
