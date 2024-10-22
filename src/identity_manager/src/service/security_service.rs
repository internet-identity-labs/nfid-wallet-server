use ic_cdk::{caller, trap};

use crate::container::container_wrapper::get_account_repo;
use crate::http::requests::DeviceType;
use crate::repository::account_repo::AccountRepoTrait;

pub fn secure_2fa() {
    let principal = caller().to_text();
    secure_principal_2fa(&principal)
}

pub fn secure_principal_2fa(principal: &String) {
    match get_account_repo().get_account() {
        None => {}
        Some(acc) => {
            if acc.is2fa_enabled {
                let requester_ap = acc.access_points.clone().iter()
                    .find(|l| l.principal_id.eq(principal))
                    .map(|l|l.device_type)
                    .expect("Failed to extract the device type for the given principal.");
                if requester_ap.eq(&DeviceType::Email) {
                    trap("Unauthorised")
                }
                if requester_ap.eq(&DeviceType::Unknown) {
                    trap("Unauthorised")
                }
            }
        }
    }
}