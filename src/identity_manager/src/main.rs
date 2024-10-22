use std::time::Duration;

use ic_cdk::{caller, trap};
use ic_cdk_macros::*;

use canister_api_macros::{admin, lambda, operator, two_f_a};
use http::response_mapper::DataResponse;
use service::{device_index_service, email_validation_service};

use crate::application_service::ApplicationService;
use crate::container::container_wrapper::{
    get_access_point_service, get_account_repo, get_account_service, get_application_service,
    get_persona_service,
};
use crate::http::requests;
use crate::http::requests::{AccountResponse, WalletVariant};
use crate::http::response_mapper;
use crate::ic_service::get_caller;
use crate::repository::account_repo::{
    Account, AccountRepo, AccountRepoTrait, ACCOUNTS, PRINCIPAL_INDEX,
};
use crate::repository::application_repo::{Application, ApplicationRepo};
use crate::repository::persona_repo::PersonaRepo;
use crate::repository::repo::{
    AdminRepo, Configuration, ConfigurationRepo, ControllersRepo, CONFIGURATION,
};
use crate::requests::{
    AccessPointRemoveRequest, AccessPointRequest, AccessPointResponse, AccountRequest,
    ConfigurationRequest, ConfigurationResponse, PersonaResponse,
};
use crate::response_mapper::{to_success_response, HttpResponse};
use crate::service::access_point_service::AccessPointServiceTrait;
use crate::service::account_service::AccountServiceTrait;
use crate::service::application_service::ApplicationServiceTrait;
use crate::service::certified_service::{get_witness, CertifiedResponse};
use crate::service::persona_service::PersonaServiceTrait;
use crate::service::security_service::{secure_2fa, secure_principal_2fa};
use crate::service::{application_service, ic_service};

mod container;
mod http;
mod logger;
mod mapper;
mod repository;
mod service;
mod structure;
mod util;

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
        token_ttl: if request.token_ttl.is_some() {
            Duration::from_secs(request.token_ttl.expect("The request.token_ttl failed after existence check."))
        } else {
            default.token_ttl
        },
        token_refresh_ttl: if request.token_ttl.is_some() {
            Duration::from_secs(request.token_refresh_ttl.expect("The request.token_refresh_ttl failed after existence check."))
        } else {
            default.token_refresh_ttl
        },
        whitelisted_phone_numbers: if request.whitelisted_phone_numbers.is_some() {
            request.whitelisted_phone_numbers.expect("The request.whitelisted_phone_numbers failed after existence check.")
        } else {
            default.whitelisted_phone_numbers
        },
        heartbeat: if request.heartbeat.is_some() {
            request.heartbeat
        } else {
            default.heartbeat
        },
        backup_canister_id: if request.backup_canister_id.is_some() {
            request.backup_canister_id
        } else {
            default.backup_canister_id
        },
        ii_canister_id: if request.ii_canister_id.is_some() {
            request.ii_canister_id.expect("The request.ii_canister_id failed after existence check.")
        } else {
            default.ii_canister_id
        },
        whitelisted_canisters: if request.whitelisted_canisters.is_some() {
            request.whitelisted_canisters
        } else {
            default.whitelisted_canisters
        },
        env: if request.env.is_some() {
            request.env
        } else {
            default.env
        },
        git_branch: if request.git_branch.is_some() {
            request.git_branch
        } else {
            default.git_branch
        },
        commit_hash: if request.commit_hash.is_some() {
            request.commit_hash
        } else {
            default.commit_hash
        },
        operator: if request.operator.is_some() {
            request.operator.expect("The request.operator failed after existence check.")
        } else {
            default.operator
        },
    };
    CONFIGURATION.with(|config| {
        config.replace(configuration);
    });
}

#[query]
async fn get_config() -> ConfigurationResponse {
    let config = CONFIGURATION.with(|config| config.borrow().clone());
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
async fn create_access_point(
    access_point_request: AccessPointRequest,
) -> HttpResponse<Vec<AccessPointResponse>> {
    let access_point_service = get_access_point_service();
    let response = access_point_service
        .create_access_point(access_point_request.clone())
        .await;
    response
}

#[update]
#[two_f_a]
async fn update_access_point(
    access_point: AccessPointRequest,
) -> HttpResponse<Vec<AccessPointResponse>> {
    let access_point_service = get_access_point_service();
    access_point_service.update_access_point(access_point.clone())
}

#[update]
#[two_f_a]
async fn remove_access_point(
    access_point: AccessPointRemoveRequest,
) -> HttpResponse<Vec<AccessPointResponse>> {
    let access_point_service = get_access_point_service();
    access_point_service.remove_access_point(access_point)
}

#[update]
async fn create_account(account_request: AccountRequest) -> HttpResponse<AccountResponse> {
    let mut account_service = get_account_service();
    let response = account_service.create_account(account_request).await;
    response
}

#[query]
#[operator]
async fn get_account_by_anchor(
    anchor: u64,
    wallet: Option<WalletVariant>,
) -> HttpResponse<AccountResponse> {
    let mut account_service = get_account_service();
    let wv = match wallet {
        None => WalletVariant::InternetIdentity,
        Some(x) => x,
    };
    let response = account_service.get_account_by_anchor(anchor, wv);
    response
}

#[update]
#[lambda]
async fn add_email_and_principal_for_create_account_validation(
    email: String,
    principal: String,
    timestamp: u64,
) -> HttpResponse<bool> {
    email_validation_service::insert(email, principal, timestamp);
    HttpResponse::data(200, true)
}

#[query]
#[operator]
async fn get_account_by_principal(princ: String) -> HttpResponse<AccountResponse> {
    let mut account_service = get_account_service();
    let response = account_service.get_account_by_principal(princ);
    response
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
#[query]
async fn read_personas() -> HttpResponse<Vec<PersonaResponse>> {
    let persona_service = get_persona_service();
    persona_service.read_personas()
}

#[query]
async fn validate_signature(payload: Option<String>) -> (u64, Option<String>) {
    let mut account_service = get_account_service();
    match account_service.get_account() {
        None => trap("Not registered"),
        Some(account) => (account.anchor, payload),
    }
}

#[deprecated()]
#[query]
async fn read_applications() -> HttpResponse<Vec<Application>> {
    let application_service = get_application_service();
    application_service.read_applications()
}

//backup
#[query]
#[operator]
async fn get_all_accounts_json(from: u32, mut to: u32) -> String {
    let account_repo = get_account_repo();
    let mut accounts: Vec<Account> = account_repo.get_all_accounts();
    accounts.sort_by(|a, b| {
        a.base_fields
            .get_created_date()
            .cmp(&b.base_fields.get_created_date())
    });
    let len = accounts.len() as u32;
    if to > len {
        to = len;
    }
    let b = &accounts[from as usize..to as usize];
    serde_json::to_string(&b).expect("Failed to serialize the response to JSON")
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
    PRINCIPAL_INDEX.with(|index| {
        ACCOUNTS.with(|accounts| {
            let mut index = index.borrow_mut();
            let accounts = accounts.borrow();
            for acc in accounts.iter() {
                index.insert(acc.1.principal_id.clone(), acc.1.principal_id.clone());
            }
        })
    })
}

#[update]
#[operator]
async fn get_remaining_size_after_rebuild_device_index_slice_from_temp_stack(
    size: Option<u64>,
) -> u64 {
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
    account_service
        .sync_recovery_phrase_from_internet_identity(anchor)
        .await
}

#[query]
async fn get_root_certified() -> CertifiedResponse {
    let caller = caller().to_text();
    secure_principal_2fa(&caller);
    let witness = match get_witness(caller.clone()) {
        Ok(tree) => tree,
        Err(_) => Vec::default(),
    };
    let mut account_service = get_account_service();
    match account_service.get_root_id_by_principal(caller) {
        None => trap("No such ap"),
        Some(principal) => {
            let certificate =
                ic_cdk::api::data_certificate().expect("No data certificate available");

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
