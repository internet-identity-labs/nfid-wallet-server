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

/// Invoked when the canister starts.
/// Initializes the application without parameters and saves the caller to storage.
#[init]
async fn init() -> () {
    AdminRepo::save(ic_service::get_caller());
}

/// Synchronizes controllers from the management canister.
/// This ensures the canister is aware of all controllers, allowing them to function as administrators.
#[update]
async fn sync_controllers() -> Vec<String> {
    let controllers = ic_service::get_controllers().await;
    ControllersRepo::save(controllers);
    ControllersRepo::get().iter().map(|x| x.to_text()).collect()
}

/// Saves the configuration into storage.
/// This is necessary for applying the global configuration for the canister.
/// This method can only be called by an administrator.
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

/// Returns the configuration to the caller.
/// This ensures that the correct configuration is persisted.
/// The configuration contains no sensitive data and is safe to be public.
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

/// Returns a list of access points to the caller based on their principal.
#[query]
async fn read_access_points() -> HttpResponse<Vec<AccessPointResponse>> {
    let access_point_service = get_access_point_service();
    access_point_service.read_access_points()
}

/// Returns the access point used by the caller.
/// This method updates the last used access point and browser for the caller's last sign-in, enabling tracking of access point usage.
/// It requires two-factor authentication (2FA) if enabled (via passkey).
#[update]
#[two_f_a]
async fn use_access_point(browser: Option<String>) -> HttpResponse<AccessPointResponse> {
    let access_point_service = get_access_point_service();
    access_point_service.use_access_point(browser)
}

/// Creates a new access point for the caller.
/// This is necessary when a user adds a new device for signing in.
/// Two-factor authentication (2FA) is required if enabled (via passkey).
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

/// Updates the existing access point for the caller.
/// This is necessary when a user updates a device.
/// Two-factor authentication (2FA) is required if enabled (via passkey).
#[update]
#[two_f_a]
async fn update_access_point(
    access_point: AccessPointRequest,
) -> HttpResponse<Vec<AccessPointResponse>> {
    let access_point_service = get_access_point_service();
    access_point_service.update_access_point(access_point.clone())
}

/// Removes the existing access point for the caller.
/// This is necessary when a user removes a device.
/// Two-factor authentication (2FA) is required if enabled (via passkey).
#[update]
#[two_f_a]
async fn remove_access_point(
    access_point: AccessPointRemoveRequest,
) -> HttpResponse<Vec<AccessPointResponse>> {
    let access_point_service = get_access_point_service();
    access_point_service.remove_access_point(access_point)
}

/// Allows the user to create an account.
/// This is necessary for users to register and subsequently add their access points.
/// Two-factor authentication (2FA) cannot be enabled before the actual registration process.
#[update]
async fn create_account(account_request: AccountRequest) -> HttpResponse<AccountResponse> {
    let mut account_service = get_account_service();
    let response = account_service.create_account(account_request).await;
    response
}

/// Returns account information for a specified anchor.
/// This is necessary for debugging purposes.
/// Accessible only to operators.
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

/// Adds the principal ID and email address to temporary storage for email validation during account creation.
/// The TTL hashmap is utilized to keep the storage efficient.
/// Accessible only to lambda users.
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

/// Returns account information for a specified principal.
/// This is necessary for debugging purposes.
/// Accessible only to operators.
#[query]
#[operator]
async fn get_account_by_principal(princ: String) -> HttpResponse<AccountResponse> {
    let mut account_service = get_account_service();
    let response = account_service.get_account_by_principal(princ);
    response
}

/// Returns the root principal ID based on any of the access point principal IDs.
/// This is necessary to establish a connection from a device to the root user account.
#[query]
async fn get_root_by_principal(princ: String) -> Option<String> {
    let mut account_service = get_account_service();
    secure_principal_2fa(&princ);
    account_service.get_root_id_by_principal(princ)
}

/// Returns the anchor ID based on any of the access point principal IDs.
/// This is necessary to obtain the anchor ID from a device associated with the root user account.
#[query]
async fn get_anchor_by_principal(princ: String) -> Option<u64> {
    let mut account_service = get_account_service();
    secure_principal_2fa(&princ);
    account_service.get_anchor_by_principal(princ)
}

/// Enables or disables the two-factor authentication (2FA) sign-in option (passkey only).
/// Once enabled, the user cannot sign in using any other access point, including Web2 devices.
/// Two-factor authentication (2FA) is required if enabled (via passkey).
#[update]
#[two_f_a]
async fn update_2fa(state: bool) -> AccountResponse {
    let mut account_service = get_account_service();
    account_service.update_2fa(state)
}

/// Returns the account associated with the caller.
/// This is necessary for displaying the user's data in the UI.
#[query]
async fn get_account() -> HttpResponse<AccountResponse> {
    let mut account_service = get_account_service();
    account_service.get_account_response()
}

/// Removes the user account associated with the caller.
/// Two-factor authentication (2FA) is required if enabled (via passkey).
/// This method is deprecated as the flow is no longer in use.
#[update]
#[two_f_a]
#[deprecated()]
async fn remove_account() -> HttpResponse<bool> {
    let mut account_service = get_account_service();
    account_service.remove_account()
}

/// Returns a list of personas.
/// A persona is a subaccount generated for a specific application with a different derivation origin.
/// This approach has been replaced by the global account and anonymous account.
/// This method is deprecated as the flow is no longer in use.
#[deprecated()]
#[query]
async fn read_personas() -> HttpResponse<Vec<PersonaResponse>> {
    let persona_service = get_persona_service();
    persona_service.read_personas()
}

/// Returns a list of applications.
/// This approach has been replaced by the global account and anonymous account.
/// This method is deprecated as the flow is no longer in use.
#[deprecated()]
#[query]
async fn read_applications() -> HttpResponse<Vec<Application>> {
    let application_service = get_application_service();
    application_service.read_applications()
}

/// Returns all accounts within the specified range.
/// This is necessary for backup purposes.
/// Accessible only to operators.
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

/// Returns the number of created anchors.
/// This is necessary for backup purposes.
/// Accessible only to operators.
#[query]
#[operator]
async fn count_anchors() -> u64 {
    let account_repo = get_account_repo();
    let accounts = account_repo.get_all_accounts().len();
    accounts as u64
}

/// Initiates the rebuild of the access point index in the canister.
/// This is necessary for constructing the index of access point principals to the root account principal.
/// This method does not apply the calculation of the certified tree.
/// Accessible only to operators.
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

/// Applies the specified amount of accounts in a single run.
/// Due to the large number of anchors, it is not feasible to execute this in one go within the post-upgrade script.
/// This method calculates the certified tree for certified query calls.
/// Accessible only to operators.
#[update]
#[operator]
async fn get_remaining_size_after_rebuild_device_index_slice_from_temp_stack(
    size: Option<u64>,
) -> u64 {
    device_index_service::get_remaining_size_after_rebuild_index_slice_from_temp_stack(size)
}

/// Saves all accounts in a temporary stack to be processed by the `get_remaining_size_after_rebuild_device_index_slice_from_temp_stack` method.
/// This process must be completed in its entirety to fully rebuild the index.
/// Accessible only to operators.
#[update]
#[operator]
async fn save_temp_stack_to_rebuild_device_index() -> String {
    device_index_service::save_temp_stack()
}

/// Retrieves the user recovery phrase from Internet Identity.
/// This is necessary in the event of an inconsistency between their recovery phrase storage and ours.
#[update]
async fn sync_recovery_phrase_from_internet_identity(anchor: u64) -> HttpResponse<AccountResponse> {
    let account_service = get_account_service();
    account_service
        .sync_recovery_phrase_from_internet_identity(anchor)
        .await
}

/// Returns a certified response.
/// This is necessary to validate user access point principals for certification query calls.
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

/// Applies changes before the canister upgrade.
#[pre_upgrade]
fn pre_upgrade() {
    repository::repo::pre_upgrade()
}

/// Applies changes after the canister upgrade.
#[post_upgrade]
fn post_upgrade() {
    repository::repo::post_upgrade()
}

fn main() {}
