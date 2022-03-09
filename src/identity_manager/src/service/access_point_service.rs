use std::collections::HashSet;
use crate::mapper::access_point_mapper::{access_point_request_to_access_point, access_point_to_access_point_response, access_point_update_request_to_access_point};
use crate::repository::access_point_repo::{AccessPoint, AccessPointRepoTrait};
use crate::requests::{AccessPointRequest, AccessPointResponse};
use crate::response_mapper::{HttpResponse, to_error_response, to_success_response};
use ic_cdk::export::Principal;
use ic_cdk::print;
use crate::repository::encrypt::account_encrypt::encrypt;

pub trait AccessPointServiceTrait {
    fn read_access_points(&self) -> HttpResponse<Vec<AccessPointResponse>>;
    fn create_access_point(&self, access_point: AccessPointRequest) -> HttpResponse<Vec<AccessPointResponse>>;
    fn remove_access_point(&self, access_point: AccessPointRequest) -> HttpResponse<Vec<AccessPointResponse>>;
}

#[derive(Default)]
pub struct AccessPointService<T> {
    pub access_point_repo: T,
}

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

    fn create_access_point(&self, access_point_request: AccessPointRequest) -> HttpResponse<Vec<AccessPointResponse>> {
        match self.access_point_repo.get_access_points() {
            Some(mut content) => {
                let access_point = access_point_request_to_access_point(access_point_request.clone());
                if content.clone().iter()
                    .any(|x| x.principal_id == access_point.principal_id) {
                    return to_error_response("Access Point exists.");
                }
                content.insert(access_point.clone());
                self.access_point_repo.store_access_points(content.clone());
                self.access_point_repo.update_account_index(access_point.principal_id);
                let response: Vec<AccessPointResponse> = content.into_iter()
                    .map(access_point_to_access_point_response)
                    .collect();
                to_success_response(response)
            }
            None => to_error_response("Unable to find Account.")
        }
    }

    fn remove_access_point(&self, access_point_request: AccessPointRequest) -> HttpResponse<Vec<AccessPointResponse>> {
        match self.access_point_repo.get_access_points() {
            Some(content) => {
                let access_point = access_point_request_to_access_point(access_point_request.clone());
                let aps: HashSet<AccessPoint> = content.iter()
                    .filter(|x| x.principal_id != access_point.principal_id)
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

