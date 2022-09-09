use std::time::Duration;
use ic_cdk::{caller, storage, trap};

use ic_cdk_macros::*;
use service::{account_service, persona_service, phone_number_service};
use crate::account_service::{AccountService, AccountServiceTrait};
use crate::persona_service::{PersonaService, PersonaServiceTrait};
use crate::repository::persona_repo::PersonaRepo;
use crate::application_service::ApplicationService;
use crate::container::container_wrapper;
use crate::container_wrapper::{get_access_point_service, get_account_service, get_application_service, get_persona_service, get_phone_number_service, get_credential_service, get_account_repo};
use crate::repository::application_repo::{Application, ApplicationRepo};
use crate::service::application_service::ApplicationServiceTrait;
use crate::service::phone_number_service::PhoneNumberServiceTrait;

use crate::http::requests;
use crate::http::requests::{AccountResponse};
use crate::http::response_mapper;
use crate::phone_number_service::PhoneNumberService;
use crate::repository::account_repo::{Account, AccountRepo, AccountRepoTrait};
use crate::repository::repo::{AdminRepo, Configuration, ConfigurationRepo, ControllersRepo};
use crate::requests::{ConfigurationRequest, AccountRequest, TokenRequest, ValidatePhoneRequest, AccessPointResponse, AccessPointRequest, CredentialVariant, PersonaRequest, PersonaResponse, ConfigurationResponse, AccessPointRemoveRequest};
use crate::requests::AccountUpdateRequest;
use crate::response_mapper::{HttpResponse, Response, to_success_response};
use crate::service::{application_service, ic_service, replica_service};
use canister_api_macros::{log_error, replicate_account, admin, collect_metrics, admin_or_lambda};
use crate::ic_service::get_caller;
use crate::replica_service::HearthCount;
use crate::service::credential_service::CredentialServiceTrait;
use crate::service::access_point_service::AccessPointServiceTrait;
use crate::service::replica_service::AccountsToReplicate;

mod service;
mod http;
mod repository;
mod mapper;
mod util;
#[cfg(test)]
mod tests;
mod container;
mod logger;

#[init]
async fn init() -> () {
    AdminRepo::save(ic_service::get_caller());
}

#[update]
async fn sync_controllers() -> Vec<String> {
    let controllers = ic_service::get_controllers().await;
    ControllersRepo::save(controllers);
    ControllersRepo::get().iter().map(|x| x.to_text()).collect()
}

#[update]
#[admin]
#[collect_metrics]
async fn configure(request: ConfigurationRequest) -> () {
    let default = ConfigurationRepo::get_default_config();
    let configuration = Configuration {
        lambda: request.lambda.unwrap_or(default.lambda),
        token_ttl: if request.token_ttl.is_some() { Duration::from_secs(request.token_ttl.unwrap()) } else { default.token_ttl },
        token_refresh_ttl: if request.token_ttl.is_some() { Duration::from_secs(request.token_refresh_ttl.unwrap()) } else { default.token_refresh_ttl },
        whitelisted_phone_numbers: if request.whitelisted_phone_numbers.is_some() { request.whitelisted_phone_numbers.unwrap() } else { default.whitelisted_phone_numbers },
        heartbeat: if request.heartbeat.is_some() { request.heartbeat } else { default.heartbeat },
        backup_canister_id: if request.backup_canister_id.is_some() { request.backup_canister_id } else { default.backup_canister_id },
        ii_canister_id: if request.ii_canister_id.is_some() { request.ii_canister_id.unwrap() } else { default.ii_canister_id },
        whitelisted_canisters: if request.whitelisted_canisters.is_some() { request.whitelisted_canisters } else { default.whitelisted_canisters },
        env: if request.env.is_some() { request.env } else { default.env },
        git_branch: if request.git_branch.is_some() { request.git_branch } else { default.git_branch },
        commit_hash: if request.commit_hash.is_some() { request.commit_hash } else { default.commit_hash },
    };
    ConfigurationRepo::save(configuration);
}

#[query]
#[admin]
async fn get_config() -> ConfigurationResponse {
    let config = ConfigurationRepo::get().clone();
    ConfigurationResponse {
        lambda: Some(config.lambda),
        token_ttl: Some(config.token_ttl.as_secs()),
        token_refresh_ttl: Some(config.token_refresh_ttl.as_secs()),
        whitelisted_phone_numbers: Some(config.whitelisted_phone_numbers),
        heartbeat: config.heartbeat,
        backup_canister_id: config.backup_canister_id,
        ii_canister_id: Some(config.ii_canister_id),
        whitelisted_canisters: config.whitelisted_canisters,
        env: config.env,
        git_branch: config.git_branch,
        commit_hash: config.commit_hash,
    }
}

#[query]
#[admin]
async fn anchors() -> HttpResponse<Vec<u64>> {
    let mut account_service = get_account_service();

    let anchors = account_service.get_all_accounts()
        .iter()
        .map(|a| a.anchor)
        .collect();

    HttpResponse {
        data: Some(anchors),
        error: None,
        status_code: 200,
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
#[collect_metrics]
async fn use_access_point() -> HttpResponse<AccessPointResponse> {
    let access_point_service = get_access_point_service();
    access_point_service.use_access_point()
}

#[update]
#[replicate_account]
#[log_error]
#[collect_metrics]
async fn create_access_point(access_point_request: AccessPointRequest) -> HttpResponse<Vec<AccessPointResponse>> {
    let access_point_service = get_access_point_service();
    let response = access_point_service.create_access_point(access_point_request.clone()).await;
    response
}

#[update]
#[replicate_account]
#[log_error]
#[collect_metrics]
async fn update_access_point(access_point: AccessPointRequest) -> HttpResponse<Vec<AccessPointResponse>> {
    let access_point_service = get_access_point_service();
    access_point_service.update_access_point(access_point.clone())
}

#[update]
#[replicate_account]
#[log_error]
#[collect_metrics]
async fn remove_access_point(access_point: AccessPointRemoveRequest) -> HttpResponse<Vec<AccessPointResponse>> {
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
    let response = account_service.create_account(account_request.clone()).await;
    response
}

#[update]
#[log_error]
#[collect_metrics]
async fn recover_account(anchor: u64) -> HttpResponse<AccountResponse> {
    let mut account_service = get_account_service();
    account_service.recover_account(anchor).await
}

#[query]
#[admin_or_lambda]
async fn get_account_by_anchor(anchor: u64) -> HttpResponse<AccountResponse> {
    let mut account_service = get_account_service();
    let response = account_service.get_account_by_anchor(anchor);
    response
}

#[query]
#[admin_or_lambda]
async fn get_account_by_principal(princ: String) -> HttpResponse<AccountResponse> {
    let mut account_service = get_account_service();
    let response = account_service.get_account_by_principal(princ);
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
    account_service.get_account_response()
}

#[query]
#[admin]
#[log_error]
async fn certify_phone_number_sha2(principal_id: String, domain: String) -> HttpResponse<String> {
    let account_service = get_account_service();
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
#[admin]
async fn remove_account_by_principal(princ: String) -> HttpResponse<bool> {
    let mut account_service = get_account_service();
    account_service.remove_account_by_principal(princ)
}

#[update]
#[log_error]
#[replicate_account]
async fn create_persona(persona: PersonaRequest) -> HttpResponse<AccountResponse> {
    let persona_service = get_persona_service();
    persona_service.create_persona(persona)
}

#[update]
#[log_error]
#[replicate_account]
async fn update_persona(persona: PersonaRequest) -> HttpResponse<AccountResponse> {
    let persona_service = get_persona_service();
    persona_service.update_persona(persona)
}

#[query]
async fn read_personas() -> HttpResponse<Vec<PersonaResponse>> {
    let persona_service = get_persona_service();
    persona_service.read_personas()
}

#[query]
async fn validate_signature(payload: Option<String>) -> (u64, Option<String>) {
    let mut account_service = get_account_service();
    match account_service.get_account() {
        None => {
            trap("Not registered")
        }
        Some(account) => {
            (account.anchor, payload)
        }
    }
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
async fn update_application(app: Application) -> HttpResponse<Vec<Application>> {
    let application_service = get_application_service();
    application_service.update_application(app)
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
async fn get_application(domain:String) -> HttpResponse<Application> {
    let application_service = get_application_service();
    application_service.get_application_by_domain(domain)
}

#[update]
async fn update_application_alias(domain: String, new_alias: String, new_name: Option<String>) -> HttpResponse<bool> {
    let application_service = get_application_service();
    application_service.update_application_alias(domain, new_alias, new_name)
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

#[query]
#[admin]
async fn get_all_accounts_json(from: u32, mut to: u32) -> String {
    let account_repo = get_account_repo();
    let mut accounts: Vec<Account> = account_repo.get_all_accounts();
    accounts.sort_by(|a, b| a.base_fields.get_created_date().cmp(&b.base_fields.get_created_date()));
    let len = accounts.len() as u32;
    if to > len {
        to = len;
    }
    let b = &accounts[from as usize..to as usize];
    serde_json::to_string(&b).unwrap()
}

#[query]
#[admin]
async fn count_anchors() -> u64 {
    let account_repo = get_account_repo();
    let accounts = account_repo.get_all_accounts().len();
    accounts as u64
}

#[update]
#[admin]
async fn add_all_accounts_json(accounts_json: String) {
    let account_repo = get_account_repo();
    let accounts: Vec<Account> = serde_json::from_str(&accounts_json).unwrap();
    account_repo.store_accounts(accounts);
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


