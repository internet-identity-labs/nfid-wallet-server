use std::collections::HashSet;
use ic_cdk::export::Principal;
use crate::{AccessPointRemoveRequest, ic_service};
use crate::mapper::access_point_mapper::{access_point_request_to_access_point, access_point_to_access_point_response};
use crate::repository::access_point_repo::{AccessPoint, AccessPointRepoTrait};
use crate::requests::{AccessPointRequest, AccessPointResponse};
use crate::response_mapper::{HttpResponse, to_error_response, to_success_response};


pub trait AccessPointServiceTrait {
    fn read_access_points(&self) -> HttpResponse<Vec<AccessPointResponse>>;
    fn use_access_point(&self) -> HttpResponse<AccessPointResponse>;
    fn create_access_point(&self, access_point: AccessPointRequest) -> HttpResponse<Vec<AccessPointResponse>>;
    fn update_access_point(&self, access_point_request: AccessPointRequest) -> HttpResponse<Vec<AccessPointResponse>>;
    fn remove_access_point(&self, access_point: AccessPointRemoveRequest) -> HttpResponse<Vec<AccessPointResponse>>;
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

    fn use_access_point(&self) -> HttpResponse<AccessPointResponse> {
        let principal = ic_service::get_caller().to_text();
        match self.access_point_repo.use_access_point(principal, ic_service::get_time()) {
            Some(access_point) => {
                to_success_response(access_point_to_access_point_response(access_point))
            }
            None => to_error_response("Unable to find object.")
        }
    }

    fn create_access_point(&self, access_point_request: AccessPointRequest) -> HttpResponse<Vec<AccessPointResponse>> {
        match self.access_point_repo.get_access_points() {
            Some(mut content) => {
                let access_point = access_point_request_to_access_point(access_point_request.clone());
                if content.clone().iter()
                    .any(|x| x.eq(&access_point)) {
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
                self.access_point_repo.update_account_index(access_point.principal_id);
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
                let principal = Principal::self_authenticating(access_point_request.pub_key).to_text();
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

