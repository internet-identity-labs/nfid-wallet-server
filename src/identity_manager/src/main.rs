use std::convert::TryInto;
use std::time::Duration;

use ic_cdk::{caller, storage, trap};
use ic_cdk::export::Principal;
use ic_cdk_macros::*;

use canister_api_macros::{admin, lambda, operator, two_f_a};
use http::requests::PrincipalEmailRequest;
use http::response_mapper::DataResponse;
use service::{account_service, email_validation_service, persona_service, device_index_service};

use crate::account_service::{AccountService, AccountServiceTrait};
use crate::application_service::ApplicationService;
use crate::container::container_wrapper;
use crate::container_wrapper::{get_access_point_service, get_account_repo, get_account_service, get_application_service, get_persona_service};
use crate::http::requests;
use crate::http::requests::{AccountResponse, WalletVariant};
use crate::http::response_mapper;
use crate::ic_service::get_caller;
use crate::persona_service::{PersonaService, PersonaServiceTrait};
use crate::replica_service::HearthCount;
use crate::repository::account_repo::{Account, AccountRepo, AccountRepoTrait, PrincipalIndex};
use crate::repository::application_repo::{Application, ApplicationRepo};
use crate::repository::persona_repo::PersonaRepo;
use crate::repository::repo::{AdminRepo, Configuration, ConfigurationRepo, ControllersRepo};
use crate::requests::{AccessPointRemoveRequest, AccessPointRequest, AccessPointResponse, AccountRequest, ConfigurationRequest, ConfigurationResponse, CredentialVariant, PersonaRequest, PersonaResponse, TokenRequest, ValidatePhoneRequest};
use crate::requests::AccountUpdateRequest;
use crate::response_mapper::{HttpResponse, to_success_response};
use crate::service::{application_service, ic_service, replica_service};
use crate::service::access_point_service::AccessPointServiceTrait;
use crate::service::application_service::ApplicationServiceTrait;
use crate::service::certified_service::{CertifiedResponse, get_witness, TREE};
use crate::service::replica_service::AccountsToReplicate;
use crate::service::security_service::{secure_2fa, secure_principal_2fa};

mod service;
mod http;
mod repository;
mod mapper;
mod util;
mod container;
mod logger;
mod structure;

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
async fn configure(request: ConfigurationRequest) -> () {
    let default = ConfigurationRepo::get_default_config();
    let configuration = Configuration {
        lambda_url: request.lambda_url.unwrap_or(default.lambda_url),
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
        operator: if request.operator.is_some() { request.operator.unwrap() } else { default.operator },
    };
    ConfigurationRepo::save(configuration);
}

#[query]
async fn get_config() -> ConfigurationResponse {
    let config = ConfigurationRepo::get().clone();
    ConfigurationResponse {
        lambda_url: Some(config.lambda_url),
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
        operator: Some(config.operator),
    }
}


#[query]
async fn read_access_points() -> HttpResponse<Vec<AccessPointResponse>> {
    let access_point_service = get_access_point_service();
    access_point_service.read_access_points()
}

#[update]
#[two_f_a]
async fn use_access_point(browser: Option<String>) -> HttpResponse<AccessPointResponse> {
    let access_point_service = get_access_point_service();
    access_point_service.use_access_point(browser)
}

#[update]
#[two_f_a]
async fn create_access_point(access_point_request: AccessPointRequest) -> HttpResponse<Vec<AccessPointResponse>> {
    let access_point_service = get_access_point_service();
    let response = access_point_service.create_access_point(access_point_request.clone()).await;
    response
}

#[update]
#[two_f_a]
async fn update_access_point(access_point: AccessPointRequest) -> HttpResponse<Vec<AccessPointResponse>> {
    let access_point_service = get_access_point_service();
    access_point_service.update_access_point(access_point.clone())
}

#[update]
#[two_f_a]
async fn remove_access_point(access_point: AccessPointRemoveRequest) -> HttpResponse<Vec<AccessPointResponse>> {
    let access_point_service = get_access_point_service();
    access_point_service.remove_access_point(access_point)
}

#[update]
async fn create_account(account_request: AccountRequest) -> HttpResponse<AccountResponse> {
    let mut account_service = get_account_service();
    let response = account_service.create_account(account_request).await;
    response
}

#[update]
async fn recover_account(anchor: u64, wallet: Option<WalletVariant>) -> HttpResponse<AccountResponse> {
    let mut account_service = get_account_service();
    account_service.recover_account(anchor, wallet).await
}


#[update]
#[lambda]
async fn add_email_and_principal_for_create_account_validation(email: String, principal: String, timestamp: u64) -> HttpResponse<bool> {
    email_validation_service::insert(email, principal, timestamp);
    HttpResponse::data(200, true)
}

#[query]
async fn get_root_by_principal(princ: String) -> Option<String> {
    let mut account_service = get_account_service();
    secure_principal_2fa(&princ);
    account_service.get_root_id_by_principal(princ)
}

#[query]
async fn get_anchor_by_principal(princ: String) -> Option<u64> {
    let mut account_service = get_account_service();
    secure_principal_2fa(&princ);
    account_service.get_anchor_by_principal(princ)
}


#[update]
#[two_f_a]
async fn update_2fa(state: bool) -> AccountResponse {
    let mut account_service = get_account_service();
    account_service.update_2fa(state)
}

#[update]
#[two_f_a]
async fn update_account(account_request: AccountUpdateRequest) -> HttpResponse<AccountResponse> {
    let mut account_service = get_account_service();
    account_service.update_account(account_request).await
}

#[query]
async fn get_account() -> HttpResponse<AccountResponse> {
    let mut account_service = get_account_service();
    account_service.get_account_response()
}

#[update]
#[two_f_a]
#[deprecated()]
async fn remove_account() -> HttpResponse<bool> {
    let mut account_service = get_account_service();
    account_service.remove_account()
}

#[deprecated()]
#[update]
#[two_f_a]
async fn create_persona(persona: PersonaRequest) -> HttpResponse<AccountResponse> {
    let persona_service = get_persona_service();
    persona_service.create_persona(persona)
}

#[deprecated()]
#[update]
#[two_f_a]
async fn update_persona(persona: PersonaRequest) -> HttpResponse<AccountResponse> {
    let persona_service = get_persona_service();
    persona_service.update_persona(persona)
}

#[deprecated()]
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

#[deprecated()]
#[query]
async fn read_applications() -> HttpResponse<Vec<Application>> {
    let application_service = get_application_service();
    application_service.read_applications()
}

#[deprecated()]
#[query]
async fn get_application(domain: String) -> HttpResponse<Application> {
    let application_service = get_application_service();
    application_service.get_application_by_domain(domain)
}

#[deprecated()]
#[update]
async fn update_application_alias(domain: String, new_alias: String, new_name: Option<String>) -> HttpResponse<bool> {
    let application_service = get_application_service();
    application_service.update_application_alias(domain, new_alias, new_name)
}

#[deprecated()]
#[query]
async fn is_over_the_application_limit(domain: String) -> HttpResponse<bool> {
    let application_service = get_application_service();
    application_service.is_over_the_application_limit(&domain)
}


//backup
#[query]
#[operator]
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
#[operator]
async fn count_anchors() -> u64 {
    let account_repo = get_account_repo();
    let accounts = account_repo.get_all_accounts().len();
    accounts as u64
}

#[update]
#[operator]
async fn rebuild_index() {
    let all_accs = get_account_service().get_all_accounts();
    let mut princ = storage::get_mut::<PrincipalIndex>();
    for acc in all_accs {
        princ.insert(acc.principal_id.clone(), acc.principal_id.clone());
    }
}

#[update]
#[operator]
async fn get_remaining_size_after_rebuild_device_index_slice_from_temp_stack(size: Option<u64>) -> u64 {
    device_index_service::get_remaining_size_after_rebuild_index_slice_from_temp_stack(size)
}

#[update]
#[operator]
async fn save_temp_stack_to_rebuild_device_index() -> String {
    device_index_service::save_temp_stack()
}

#[update]
async fn sync_recovery_phrase_from_internet_identity(anchor: u64) -> HttpResponse<AccountResponse> {
    let account_service = get_account_service();
    account_service.sync_recovery_phrase_from_internet_identity(anchor).await
}

#[query]
async fn get_root_certified() -> CertifiedResponse {
    let caller = caller().to_text();
    secure_principal_2fa(&caller);
    let witness = match get_witness(caller.clone()) {
        Ok(tree) => tree,
        Err(_) => {
            Vec::default()
        }
    };
    let mut account_service = get_account_service();
    match account_service.get_root_id_by_principal(caller) {
        None => {
            trap("No such ap")
        }
        Some(principal) => {
            let certificate = ic_cdk::api::data_certificate().expect("No data certificate available");

            CertifiedResponse {
                response: principal,
                certificate,
                witness,
            }
        }
    }
}

#[pre_upgrade]
fn pre_upgrade() {
    repository::repo::pre_upgrade()
}

#[post_upgrade]
fn post_upgrade() {
    repository::repo::post_upgrade()
}

//some test comment to change hash
fn main() {}


