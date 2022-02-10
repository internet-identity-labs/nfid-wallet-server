use std::time::Duration;
use std::any::Any;
use std::cell::RefCell;
use std::sync::Arc;
use ic_cdk::export::Principal;
use ic_cdk::{print, trap};
use ic_cdk_macros::*;
use inject::{container, get};

use service::{access_point_service, account_service, persona_service, phone_number_service};
use repository::repo::{AccessPoint, AdminRepo, Application, Configuration, ConfigurationRepo};
use crate::account_service::{AccountService, AccountServiceTrait};
use crate::persona_service::{PersonaService, PersonaServiceTrait};
use crate::access_point_service::{AccessPointService, AccessPointServiceTrait};
use crate::repository::access_point_repo::AccessPointRepo;
use crate::repository::persona_repo::PersonaRepo;
use crate::application_service::ApplicationService;
use crate::container::container_wrapper;
use crate::container_wrapper::{get_access_point_service, get_account_service, get_application_service, get_persona_service, get_phone_number_service};
use crate::repository::application_repo::ApplicationRepo;
use crate::service::application_service::ApplicationServiceTrait;
use crate::service::phone_number_service::PhoneNumberServiceTrait;

use crate::http::requests;
use crate::http::requests::{AccountResponse, PersonaVariant};
use crate::http::response_mapper;
use crate::phone_number_service::PhoneNumberService;
use crate::repository::account_repo::AccountRepo;
use crate::requests::{ConfigurationRequest, HTTPAccountRequest, HTTPVerifyPhoneNumberRequest};
use crate::requests::HTTPAccountUpdateRequest;
use crate::response_mapper::{HttpResponse, unauthorized};
use crate::service::{application_service, ic_service};

mod service;
mod http;
mod repository;
mod mapper;
mod util;
mod tests;
mod container;

#[init]
async fn init() -> () {
    AdminRepo::save(ic_service::get_caller());
}

#[update]
async fn configure(request: ConfigurationRequest) -> () {
    if !AdminRepo::get().eq(&ic_service::get_caller()) {
        trap("Unauthorized")
    }
    let configuration = Configuration {
        lambda: request.lambda,
        token_ttl: Duration::from_secs(request.token_ttl),
        token_refresh_ttl: Duration::from_secs(request.token_refresh_ttl),
        key: request.key,
        whitelisted: request.whitelisted_phone_numbers.unwrap_or(Vec::default()),
    };

    ConfigurationRepo::save(configuration);
}

#[update]
async fn validate_phone_number(phone_number: String) -> HttpResponse<bool> {
    let phone_number_service = get_phone_number_service();
    phone_number_service.validate_phone_number(phone_number)
}

#[update]
async fn post_token(request: HTTPVerifyPhoneNumberRequest) -> HttpResponse<bool> {
    let mut phone_number_service = get_phone_number_service();
    phone_number_service.post_token(request)
}

#[update]
async fn create_account(account_request: HTTPAccountRequest) -> HttpResponse<AccountResponse> {
    let mut account_service = get_account_service();
    account_service.create_account(account_request)
}

#[update]
async fn update_account(account_request: HTTPAccountUpdateRequest) -> HttpResponse<AccountResponse> {
    let mut account_service = get_account_service();
    account_service.update_account(account_request)
}

#[query]
async fn get_account() -> HttpResponse<AccountResponse> {
    let mut account_service = get_account_service();
    account_service.get_account()
}

#[update]
async fn read_access_points() -> HttpResponse<Vec<AccessPoint>> {
    let mut access_point_service = get_access_point_service();
    access_point_service.read_access_points()
}

#[update]
async fn create_access_point(access_point: AccessPoint) -> HttpResponse<Vec<AccessPoint>> {
    let mut access_point_service = get_access_point_service();
    access_point_service.create_access_point(access_point)
}

#[update]
async fn update_access_point(access_point: AccessPoint) -> HttpResponse<Vec<AccessPoint>> {
    let mut access_point_service = get_access_point_service();
    access_point_service.update_access_point(access_point)
}

#[update]
async fn remove_access_point(access_point: AccessPoint) -> HttpResponse<Vec<AccessPoint>> {
    let mut access_point_service = get_access_point_service();
    access_point_service.remove_access_point(access_point)
}

#[update]
async fn create_persona(persona: PersonaVariant) -> HttpResponse<AccountResponse> {
    let mut persona_service = get_persona_service();
    persona_service.create_persona(persona)
}

#[update]
async fn read_personas() -> HttpResponse<Vec<PersonaVariant>> {
    let mut persona_service = get_persona_service();
    persona_service.read_personas()
}

#[update]
async fn create_application(app: Application) -> HttpResponse<Vec<Application>> {
    let application_service = get_application_service();
    application_service.create_application(app)
}

#[update]
async fn delete_application(app: String) -> HttpResponse<bool> {
    let application_service = get_application_service();
    application_service.delete_application(app)
}

#[query]
async fn read_applications() -> HttpResponse<Vec<Application>> {
    let application_service = get_application_service();
    application_service.read_applications()
}

#[query]
async fn is_over_the_application_limit(domain: String) -> HttpResponse<bool> {
    let application_service = get_application_service();
    application_service.is_over_the_application_limit(&domain)
}

#[pre_upgrade]
fn pre_upgrade() {
    repository::repo::pre_upgrade()
}

#[post_upgrade]
fn post_upgrade() {
    repository::repo::post_upgrade()
}

fn main() {}


