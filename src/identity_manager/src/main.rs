use std::time::Duration;
use ic_cdk::{caller, storage, trap};
use ic_cdk::export::Principal;

use ic_cdk_macros::*;
use service::{account_service, persona_service, phone_number_service};
use crate::account_service::{AccountService, AccountServiceTrait};
use crate::persona_service::{PersonaService, PersonaServiceTrait};
use crate::repository::persona_repo::PersonaRepo;
use crate::application_service::ApplicationService;
use crate::container::container_wrapper;
use crate::container_wrapper::{get_access_point_service, get_account_service, get_application_service, get_persona_service, get_phone_number_service, get_credential_service};
use crate::repository::application_repo::{Application, ApplicationRepo};
use crate::service::application_service::ApplicationServiceTrait;
use crate::service::phone_number_service::PhoneNumberServiceTrait;

use crate::http::requests;
use crate::http::requests::{AccountResponse};
use crate::http::response_mapper;
use crate::phone_number_service::PhoneNumberService;
use crate::repository::account_repo::{Account, AccountRepo};
use crate::repository::repo::{AdminRepo, Configuration, ConfigurationRepo};
use crate::requests::{ConfigurationRequest, AccountRequest, TokenRequest, ValidatePhoneRequest, AccessPointResponse, AccessPointRequest, CredentialVariant, PersonaRequest, PersonaResponse, ConfigurationResponse};
use crate::requests::AccountUpdateRequest;
use crate::response_mapper::{HttpResponse, Response, to_success_response};
use crate::service::{application_service, ic_service, replica_service};
use canister_api_macros::{log_error, replicate_account, admin, collect_metrics};
use crate::ic_service::get_caller;
use crate::logger::logger::{Log, LogLevel, LogRepo};
use crate::replica_service::HearthCount;
use crate::service::credential_service::CredentialServiceTrait;
use crate::service::access_point_service::AccessPointServiceTrait;
use crate::service::replica_service::AccountsToReplicate;

mod service;
mod http;
mod repository;
mod mapper;
mod util;
mod tests;
mod container;
mod logger;

#[init]
async fn init() -> () {
    AdminRepo::save(ic_service::get_caller());
}

#[update]
#[admin]
#[collect_metrics]
async fn configure(request: ConfigurationRequest) -> () {
    let configuration = Configuration {
        lambda: request.lambda,
        token_ttl: Duration::from_secs(request.token_ttl),
        token_refresh_ttl: Duration::from_secs(request.token_refresh_ttl),
        whitelisted_phone_numbers: request.whitelisted_phone_numbers.unwrap_or(Vec::default()),
        heartbeat: request.heartbeat,
        backup_canister_id: request.backup_canister_id,
        ii_canister_id: request.ii_canister_id,
        whitelisted_canisters: request.whitelisted_canisters,
        env: request.env,
        git_branch: request.git_branch,
        commit_hash: request.commit_hash,
    };
    ConfigurationRepo::save(configuration);
}

#[query]
#[admin]
async fn get_config() -> ConfigurationResponse {
    let config = ConfigurationRepo::get().clone();
    ConfigurationResponse {
        lambda: config.lambda,
        token_ttl: config.token_ttl.as_secs(),
        token_refresh_ttl: config.token_refresh_ttl.as_secs(),
        whitelisted_phone_numbers: Some(config.whitelisted_phone_numbers),
        heartbeat: config.heartbeat,
        backup_canister_id: config.backup_canister_id,
        ii_canister_id: config.ii_canister_id,
        whitelisted_canisters: config.whitelisted_canisters,
        env: config.env,
        git_branch: config.git_branch,
        commit_hash: config.commit_hash,
    }
}

#[query]
async fn credentials() -> HttpResponse<Vec<CredentialVariant>> {
    let credential_service = get_credential_service();
    credential_service.credentials()
}

#[query]
async fn read_access_points() -> HttpResponse<Vec<AccessPointResponse>> {
    let access_point_service = get_access_point_service();
    access_point_service.read_access_points()
}

#[update]
#[replicate_account]
#[log_error]
#[collect_metrics]
async fn create_access_point(access_point: AccessPointRequest) -> HttpResponse<Vec<AccessPointResponse>> {
    let access_point_service = get_access_point_service();
    let response = access_point_service.create_access_point(access_point.clone());
    if response.error.is_none() {
        ic_service::trap_if_not_authenticated(get_account_service().get_account().data.unwrap().anchor,
                                              Principal::self_authenticating(access_point.pub_key.clone())).await;
    }
    response
}

#[update]
#[replicate_account]
#[log_error]
#[collect_metrics]
async fn remove_access_point(access_point: AccessPointRequest) -> HttpResponse<Vec<AccessPointResponse>> {
    let access_point_service = get_access_point_service();
    access_point_service.remove_access_point(access_point)
}

#[update]
#[replicate_account]
#[log_error]
#[collect_metrics]
async fn verify_token(token: String) -> Response {
    let phone_number_service = get_phone_number_service();
    phone_number_service.verify_token(token)
}

#[update]
#[log_error]
#[collect_metrics]
async fn validate_phone(request: ValidatePhoneRequest) -> Response {
    let phone_number_service = get_phone_number_service();
    phone_number_service.validate_phone(request)
}

#[update]
#[log_error]
#[collect_metrics]
async fn post_token(request: TokenRequest) -> Response {
    let phone_number_service = get_phone_number_service();
    phone_number_service.post_token(request)
}

#[update]
#[log_error]
#[replicate_account]
#[collect_metrics]
async fn create_account(account_request: AccountRequest) -> HttpResponse<AccountResponse> {
    let mut account_service = get_account_service();
    let response = account_service.create_account(account_request.clone());
    if response.error.is_none() {  //todo migrate to macros
        ic_service::trap_if_not_authenticated(account_request.anchor.clone(), get_caller()).await;
    }
    response
}

#[update]
#[log_error]
#[collect_metrics]
async fn recover_account(anchor: u64) -> HttpResponse<AccountResponse> {
    let mut account_service = get_account_service();
    ic_service::trap_if_not_authenticated(anchor.clone(), get_caller()).await; //todo add mock II server for autotests
    let response = account_service.recover_account(anchor);
    response
}

#[update]
#[log_error]
#[replicate_account]
#[collect_metrics]
async fn update_account(account_request: AccountUpdateRequest) -> HttpResponse<AccountResponse> {
    let mut account_service = get_account_service();
    account_service.update_account(account_request)
}

#[query]
async fn get_account() -> HttpResponse<AccountResponse> {
    let mut account_service = get_account_service();
    account_service.get_account()
}

#[query]
#[admin]
#[log_error]
async fn certify_phone_number_sha2(principal_id: String, domain: String) -> HttpResponse<String> {
    let mut account_service = get_account_service();
    account_service.certify_phone_number_sha2(principal_id, domain)
}

#[update]
#[log_error]
async fn remove_account() -> HttpResponse<bool> {
    let mut account_service = get_account_service();
    account_service.remove_account()
}

#[update]
#[log_error]
#[replicate_account]
async fn create_persona(persona: PersonaRequest) -> HttpResponse<AccountResponse> {
    let persona_service = get_persona_service();
    persona_service.create_persona(persona)
}

#[query]
async fn read_personas() -> HttpResponse<Vec<PersonaResponse>> {
    let persona_service = get_persona_service();
    persona_service.read_personas()
}

#[update]
#[log_error]
#[admin]
#[collect_metrics]
async fn create_application(app: Application) -> HttpResponse<Vec<Application>> {
    let application_service = get_application_service();
    application_service.create_application(app)
}

#[update]
#[log_error]
#[admin]
#[collect_metrics]
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
pub async fn get_logs(n: usize) -> Vec<Log> {
    LogRepo::get(n)
}

#[query]
pub async fn get_all_logs() -> Vec<Log> {
    LogRepo::get_all()
}

#[query]
async fn is_over_the_application_limit(domain: String) -> HttpResponse<bool> {
    let application_service = get_application_service();
    application_service.is_over_the_application_limit(&domain)
}

#[heartbeat]
async fn heartbeat_function() {
    if ConfigurationRepo::exists() && ConfigurationRepo::get().heartbeat.is_some() {
        let i = storage::get_mut::<HearthCount>();
        if (*i % ConfigurationRepo::get().heartbeat.unwrap()) == 0 && !storage::get_mut::<AccountsToReplicate>().is_empty() {
            flush_account().await;
        }
        *i += 1;
    }
}

#[update]
#[log_error]
async fn flush_account() -> HttpResponse<bool> {
    replica_service::flush().await
}

#[update]
#[log_error]
#[admin]
async fn store_accounts(accounts: Vec<Account>) -> HttpResponse<bool> {
    let mut account_service = get_account_service();
    account_service.store_accounts(accounts);
    to_success_response(true)
}

#[update]
#[log_error]
#[admin]
async fn restore_accounts(canister_id: String) -> HttpResponse<bool> {
    replica_service::restore_and_flush(canister_id).await
}

#[pre_upgrade]
fn pre_upgrade() {
    repository::repo::pre_upgrade()
}

#[post_upgrade]
fn post_upgrade() {
    repository::repo::post_upgrade()
}

#[ic_cdk_macros::query(name = "getCanisterMetrics")]
pub async fn get_canister_metrics(parameters: canistergeek_ic_rust::api_type::GetMetricsParameters) -> Option<canistergeek_ic_rust::api_type::CanisterMetrics<'static>> {
    canistergeek_ic_rust::monitor::get_metrics(&parameters)
}

#[ic_cdk_macros::update(name = "collectCanisterMetrics")]
pub async fn collect_canister_metrics() -> () {
    canistergeek_ic_rust::monitor::collect_metrics();
}

#[ic_cdk_macros::query(name = "getCanisterLog")]
pub async fn get_canister_log(request: Option<canistergeek_ic_rust::api_type::CanisterLogRequest>) -> Option<canistergeek_ic_rust::api_type::CanisterLogResponse<'static>> {
    canistergeek_ic_rust::logger::get_canister_log(request)
}

fn main() {}


