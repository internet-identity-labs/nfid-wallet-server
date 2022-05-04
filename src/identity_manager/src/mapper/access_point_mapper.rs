use ic_cdk::export::Principal;
use crate::repository::repo::{BasicEntity};
use crate::repository::access_point_repo::AccessPoint;
use crate::requests::{AccessPointRequest, AccessPointResponse};

pub fn access_point_to_access_point_response(access_point: AccessPoint) -> AccessPointResponse {
    AccessPointResponse {
        principal_id: access_point.principal_id,
        icon: access_point.icon.unwrap_or("Unknown".to_string()),
        device: access_point.device.unwrap_or("Unknown".to_string()),
        browser: access_point.browser.unwrap_or("Unknown".to_string()),
        last_used: access_point.last_used.unwrap(),
    }
}

pub fn access_point_request_to_access_point(request: AccessPointRequest) -> AccessPoint {
    let basic = BasicEntity::new();
    AccessPoint {
        principal_id: Principal::self_authenticating(request.pub_key).to_text(),
        icon: Some(request.icon),
        device: Some(request.device),
        browser: Some(request.browser),
        last_used: Some(basic.get_created_date().clone()),
        base_fields: basic,
    }
}
