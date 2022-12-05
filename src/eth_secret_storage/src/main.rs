use std::collections::HashMap;

use canister_api_macros::{admin, collect_metrics};
use ic_cdk::export::Principal;
use ic_cdk::export::candid::CandidType;
use ic_cdk::{trap, storage};
use repository::secret_repo;
use serde::Deserialize;
use ic_cdk_macros::{init, update, pre_upgrade, post_upgrade};
use service::response_service::HttpResponse;
use service::secret_service::get_secret_by_signature;
use crate::repository::configuration_repo::{AdminRepo, ConfigurationRepo, Configuration, ControllersRepo};
use crate::service::ic_service::get_caller;
 
mod constant;
mod service;
mod repository;

#[derive(Clone, Debug, Deserialize, CandidType)]
pub struct PersistedData {
    pub admin: Option<Principal>,
    pub secrets: Option<HashMap<String, HashMap<String, String>>>
}

#[init]
async fn init() -> () {
    AdminRepo::save(get_caller());
}

#[update]
#[admin]
#[collect_metrics]
async fn configure() -> () {
    let configuration = Configuration {
        whitelisted_canisters: None,
    };
    ConfigurationRepo::save(configuration);
}

#[update]
#[collect_metrics]
async fn secret_by_signature(app: String, signature: String) -> HttpResponse<String> {
    get_secret_by_signature(app, signature).await
}

#[pre_upgrade]
fn pre_upgrade() {
    let pre_upgrade_data = PersistedData {
        admin: Some(AdminRepo::get()),
        secrets: Some(secret_repo::get_all())
    };
    match storage::stable_save((pre_upgrade_data, 0)) {
        Ok(_) => (),
        Err(message) => trap(&format!("Failed to preupgrade: {}", message))
    }
}

#[post_upgrade]
fn post_upgrade() {
    match storage::stable_restore() {
        Ok(store) => {
            let (post_data, _): (PersistedData, i32) = store;
            if post_data.admin.is_some() {
                AdminRepo::save(post_data.admin.unwrap());
                secret_repo::save_all(post_data.secrets.unwrap())
            }
        }
        Err(message) => trap(message.as_str())
    }
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