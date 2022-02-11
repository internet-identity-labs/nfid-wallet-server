use crate::repository::repo::{BasicEntity};
use crate::repository::access_point_repo::AccessPoint;
use crate::requests::{AccessPointRequest, AccessPointResponse};

pub fn access_point_to_access_point_response(access_point: AccessPoint) -> AccessPointResponse {
    AccessPointResponse {
        pub_key: access_point.pub_key,
        last_used: access_point.last_used,
        make: access_point.make,
        model: access_point.model,
        browser: access_point.browser,
        name: access_point.name,
    }
}

pub fn access_point_request_to_access_point(access_point: AccessPointRequest) -> AccessPoint {
    AccessPoint {
        pub_key: access_point.pub_key,
        last_used: access_point.last_used,
        make: access_point.make,
        model: access_point.model,
        browser: access_point.browser,
        name: access_point.name,
        base_fields: BasicEntity::new(),
    }
}

pub fn access_point_update_request_to_access_point(access_point_update_request: AccessPointRequest, mut access_point: AccessPoint) -> AccessPoint {
    access_point.pub_key = access_point_update_request.pub_key;
    access_point.last_used = access_point_update_request.last_used;
    access_point.make = access_point_update_request.make;
    access_point.model = access_point_update_request.model;
    access_point.browser = access_point_update_request.browser;
    access_point.name = access_point_update_request.name;
    access_point.base_fields.update_modified_date();
    access_point
}
