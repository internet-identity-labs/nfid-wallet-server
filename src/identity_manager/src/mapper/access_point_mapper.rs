use ic_cdk::export::Principal;
use crate::repository::repo::{BasicEntity};
use crate::repository::access_point_repo::AccessPoint;
use crate::requests::{AccessPointRequest, AccessPointResponse};

pub fn access_point_to_access_point_response(access_point: AccessPoint) -> AccessPointResponse {
    AccessPointResponse {
        principal_id: access_point.principal_id,
    }
}

pub fn access_point_request_to_access_point(access_point: AccessPointRequest) -> AccessPoint {
    AccessPoint {
        principal_id: Principal::self_authenticating(access_point.pub_key).to_text(),
        base_fields: BasicEntity::new(),
    }
}

pub fn access_point_update_request_to_access_point(access_point_update_request: AccessPointRequest, mut access_point: AccessPoint) -> AccessPoint {
    access_point.principal_id = Principal::self_authenticating(access_point_update_request.pub_key).to_text();
    access_point.base_fields.update_modified_date();
    access_point
}
