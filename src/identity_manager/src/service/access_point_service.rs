
use crate::repository::repo::{AccessPoint, AccessPointRepo};
use crate::response_mapper::{HttpResponse, to_error_response, to_success_response};

pub fn read_access_points() -> HttpResponse<Vec<AccessPoint>> {
    match AccessPointRepo::get_access_points() {
        Some(content) => { to_success_response(content) }
        None => to_error_response("Unable to find Account.")
    }
}

pub fn create_access_point(access_point: AccessPoint) -> HttpResponse<Vec<AccessPoint>> {
    match AccessPointRepo::get_access_points() {
        Some(mut content) => {
            if content.clone().iter()
                .any(|x| x.pub_key == access_point.pub_key) {
                return to_error_response("Access Point exists.");
            }
            content.push(access_point);
            AccessPointRepo::store_access_points(content.clone());
            to_success_response(content)
        }
        None => to_error_response("Unable to find Account.")
    }
}

pub fn update_access_point(access_point: AccessPoint) -> HttpResponse<Vec<AccessPoint>> {
    match AccessPointRepo::get_access_points() {
        Some(content) => {
            let mut a: Vec<AccessPoint> = content.iter()
                .filter(|x| x.pub_key != access_point.pub_key)
                .cloned()
                .collect();
            if a.len() == content.len() {
                return to_error_response("Access Point not exists.");
            }
            a.push(access_point);
            AccessPointRepo::store_access_points(a.clone());
            to_success_response(a)
        }
        None => to_error_response("Unable to find Account.")
    }
}

pub fn remove_access_point(access_point: AccessPoint) -> HttpResponse<Vec<AccessPoint>> {
    match AccessPointRepo::get_access_points() {
        Some(content) => {
            let a: Vec<AccessPoint> = content.iter()
                .filter(|x| x.pub_key != access_point.pub_key)
                .cloned()
                .collect();
            if a.len() == content.len() {
                return to_error_response("Access Point not exists.");
            }
            AccessPointRepo::store_access_points(a.clone());
            to_success_response(a)
        }
        None => to_error_response("Unable to find Account.")
    }
}

