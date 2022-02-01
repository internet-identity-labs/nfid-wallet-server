use std::cell::RefCell;
use std::time::Duration;

use blake3::Hash;
use ic_cdk::api::caller;
use ic_cdk::trap;
use ic_cdk_macros::*;

use repository::repo::AccessPoint;
use repository::repo;
use service::{account_service, access_point_service, persona_service, phone_number_service};
use structure::ttlhashmap::TtlHashMap;

use crate::http::requests;
use crate::http::requests::{AccountResponse, PersonaVariant};
use crate::http::response_mapper;
use crate::repo::{AdminRepo, ConfigurationRepo};
use crate::requests::{Configuration, HTTPAccountRequest, HTTPVerifyPhoneNumberRequest};
use crate::requests::HTTPAccountUpdateRequest;
use crate::response_mapper::{HttpResponse, unauthorized};

mod service;
mod http;
mod repository;
mod mapper;
mod structure;
mod util;
mod tests;

const DEFAULT_TOKEN_TTL: Duration = Duration::from_secs(30);

#[init]
async fn init() -> () {
    AdminRepo::save(caller());
}

#[update]
async fn configure(configuration: Configuration) -> () {
    if !AdminRepo::get().eq(&caller()) {
        trap("Unauthorized")
    }

    ConfigurationRepo::save(configuration);

    TOKEN_STORAGE.with(|token_storage| {
        let ttl = Duration::from_secs(configuration.token_ttl.clone());
        token_storage.borrow_mut().ttl = ttl;
    });
}

thread_local! {
    static TOKEN_STORAGE: RefCell<TtlHashMap<Hash, Hash>> = RefCell::new(TtlHashMap::new(DEFAULT_TOKEN_TTL));
}

#[query]
async fn validate_phone_number(phone_number: String) -> HttpResponse<bool> {
    phone_number_service::validate_phone_number(phone_number)
}

#[update]
async fn post_token(request: HTTPVerifyPhoneNumberRequest) -> HttpResponse<bool> {
    phone_number_service::post_token(request)
}

#[update]
async fn create_account(account_request: HTTPAccountRequest) -> HttpResponse<AccountResponse> {
    account_service::create_account(account_request)
}

#[update]
async fn update_account(account_request: HTTPAccountUpdateRequest) -> HttpResponse<AccountResponse> {
    account_service::update_account(account_request)
}

#[query]
async fn get_account() -> HttpResponse<AccountResponse> {
    account_service::get_account()
}

#[update]
async fn read_access_points() -> HttpResponse<Vec<AccessPoint>> {
    access_point_service::read_access_points()
}

#[update]
async fn create_access_point(access_point: AccessPoint) -> HttpResponse<Vec<AccessPoint>> {
    access_point_service::create_access_point(access_point)
}

#[update]
async fn update_access_point(access_point: AccessPoint) -> HttpResponse<Vec<AccessPoint>> {
    access_point_service::update_access_point(access_point)
}

#[update]
async fn remove_access_point(access_point: AccessPoint) -> HttpResponse<Vec<AccessPoint>> {
    access_point_service::remove_access_point(access_point)
}

#[update]
async fn create_persona(persona: PersonaVariant) -> HttpResponse<AccountResponse> {
    persona_service::create_persona(persona)
}

#[update]
async fn read_personas() -> HttpResponse<Vec<PersonaVariant>> {
    persona_service::read_personas()
}

#[pre_upgrade]
fn pre_upgrade() {
    repo::pre_upgrade()
}

#[post_upgrade]
fn post_upgrade() {
    repo::post_upgrade()
}

fn main() {}
