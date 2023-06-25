use std::collections::HashSet;

use async_trait::async_trait;
use ic_cdk::export::Principal;
use ic_cdk::{caller, trap};

use crate::{AccessPointRemoveRequest, Account, AccountServiceTrait, get_account_service, ic_service};
use crate::http::requests::{DeviceType, WalletVariant};
use crate::ic_service::DeviceData;
use crate::mapper::access_point_mapper::{access_point_request_to_access_point, access_point_to_access_point_response, recovery_device_data_to_access_point};
use crate::repository::access_point_repo::{AccessPoint, AccessPointRepoTrait};
use crate::requests::{AccessPointRequest, AccessPointResponse};
use crate::response_mapper::{HttpResponse, to_error_response, to_success_response};

#[async_trait(? Send)]
pub trait AccessPointServiceTrait {
    fn read_access_points(&self) -> HttpResponse<Vec<AccessPointResponse>>;
    fn use_access_point(&self, browser: Option<String>) -> HttpResponse<AccessPointResponse>;
    async fn create_access_point(&self, access_point: AccessPointRequest) -> HttpResponse<Vec<AccessPointResponse>>;
    fn update_access_point(&self, access_point_request: AccessPointRequest) -> HttpResponse<Vec<AccessPointResponse>>;
    fn remove_access_point(&self, access_point: AccessPointRemoveRequest) -> HttpResponse<Vec<AccessPointResponse>>;
    fn migrate_recovery_device(&self, device_data: DeviceData, account: &Account) -> Account;
}

#[derive(Default)]
pub struct AccessPointService<T> {
    pub access_point_repo: T,
}

#[async_trait(? Send)]
impl<T: AccessPointRepoTrait> AccessPointServiceTrait for AccessPointService<T> {
    fn read_access_points(&self) -> HttpResponse<Vec<AccessPointResponse>> {
        match self.access_point_repo.get_access_points() {
            Some(content) => {
                let response: Vec<AccessPointResponse> = content.into_iter()
                    .map(access_point_to_access_point_response)
                    .collect();
                to_success_response(response)
            }
            None => to_error_response("Unable to find Account.")
        }
    }

    fn use_access_point(&self, browser: Option<String>) -> HttpResponse<AccessPointResponse> {
        let principal = ic_service::get_caller().to_text();
        match self.access_point_repo.use_access_point(principal, ic_service::get_time(), browser) {
            Some(access_point) => {
                to_success_response(access_point_to_access_point_response(access_point))
            }
            None => to_error_response("Unable to find object.")
        }
    }

    async fn create_access_point(&self, access_point_request: AccessPointRequest) -> HttpResponse<Vec<AccessPointResponse>> {
        match get_account_service().get_account() {
            Some(acc) => {
                let mut access_points = acc.access_points;
                let princ = Principal::from_text(access_point_request.pub_key.clone()).unwrap();
                match acc.wallet {
                    WalletVariant::NFID => {
                        if !acc.wallet.eq(&WalletVariant::NFID) {
                            trap("Unable to add access point")
                        }
                    }
                    WalletVariant::InternetIdentity => {
                        ic_service::trap_if_not_authenticated(acc.anchor, princ).await;
                    }
                }
                let access_point = access_point_request_to_access_point(access_point_request.clone());
                if access_points.clone().iter()
                    .any(|x| x.eq(&access_point)) {
                    return to_error_response("Access Point exists.");
                }
                access_points.insert(access_point.clone());
                self.access_point_repo.store_access_points_by_principal(access_points.clone(), acc.principal_id.clone());
                self.access_point_repo.update_account_index(access_point.principal_id, acc.principal_id);
                let response: Vec<AccessPointResponse> = access_points.into_iter()
                    .map(access_point_to_access_point_response)
                    .collect();
                to_success_response(response)
            }
            None => to_error_response("Unable to find Account.")
        }
    }

    fn migrate_recovery_device(&self, device_data: DeviceData, account: &Account) -> Account {
        let mut devices = HashSet::new();
        let ap = recovery_device_data_to_access_point(device_data);
        let princ = ap.principal_id.clone();
        devices.insert(ap);
        let acc = self.access_point_repo.store_access_points_by_principal(devices, account.principal_id.clone());
        self.access_point_repo.update_account_index(princ, account.principal_id.clone());
        acc.unwrap()
    }

    fn update_access_point(&self, access_point_request: AccessPointRequest) -> HttpResponse<Vec<AccessPointResponse>> {
        match self.access_point_repo.get_access_points() {
            Some(mut content) => {
                let access_point = access_point_request_to_access_point(access_point_request.clone());
                if !content.clone().iter()
                    .any(|x| x.eq(&access_point)) {
                    return to_error_response("Access Point not exists.");
                }
                content.replace(access_point.clone());
                self.access_point_repo.store_access_points(content.clone());
                let response: Vec<AccessPointResponse> = content.into_iter()
                    .map(access_point_to_access_point_response)
                    .collect();
                to_success_response(response)
            }
            None => to_error_response("Unable to find Account.")
        }
    }

    fn remove_access_point(&self, access_point_request: AccessPointRemoveRequest) -> HttpResponse<Vec<AccessPointResponse>> {
        match self.access_point_repo.get_access_points() {
            Some(content) => {
                let principal = access_point_request.pub_key;
                let aps: HashSet<AccessPoint> = content.iter()
                    .filter(|x| x.principal_id != principal)
                    .cloned()
                    .collect();
                if aps.len() == content.len() {
                    return to_error_response("Access Point not exists.");
                }
                self.access_point_repo.store_access_points(aps.clone());
                let response: Vec<AccessPointResponse> = aps.into_iter()
                    .map(access_point_to_access_point_response)
                    .collect();
                to_success_response(response)
            }
            None => to_error_response("Unable to find Account.")
        }
    }
}

