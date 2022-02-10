use crate::repository::access_point_repo::AccessPointRepoTrait;
use crate::repository::repo::{AccessPoint};
use crate::response_mapper::{HttpResponse, to_error_response, to_success_response};


pub trait AccessPointServiceTrait {
    fn read_access_points(&self) -> HttpResponse<Vec<AccessPoint>>;
    fn create_access_point(&self, access_point: AccessPoint) -> HttpResponse<Vec<AccessPoint>>;
    fn update_access_point(&self, access_point: AccessPoint) -> HttpResponse<Vec<AccessPoint>>;
    fn remove_access_point(&self, access_point: AccessPoint) -> HttpResponse<Vec<AccessPoint>>;
}

#[derive(Default)]
pub struct AccessPointService<T> {
    pub access_point_repo: T,
}

impl<T: AccessPointRepoTrait> AccessPointServiceTrait for AccessPointService<T> {
    fn read_access_points(&self) -> HttpResponse<Vec<AccessPoint>> {
        match self.access_point_repo.get_access_points() {
            Some(content) => { to_success_response(content) }
            None => to_error_response("Unable to find Account.")
        }
    }

    fn create_access_point(&self, access_point: AccessPoint) -> HttpResponse<Vec<AccessPoint>> {
        match self.access_point_repo.get_access_points() {
            Some(mut content) => {
                if content.clone().iter()
                    .any(|x| x.pub_key == access_point.pub_key) {
                    return to_error_response("Access Point exists.");
                }
                content.push(access_point);
                self.access_point_repo.store_access_points(content.clone());
                to_success_response(content)
            }
            None => to_error_response("Unable to find Account.")
        }
    }

    fn update_access_point(&self, access_point: AccessPoint) -> HttpResponse<Vec<AccessPoint>> {
        match self.access_point_repo.get_access_points() {
            Some(content) => {
                let mut a: Vec<AccessPoint> = content.iter()
                    .filter(|x| x.pub_key != access_point.pub_key)
                    .cloned()
                    .collect();
                if a.len() == content.len() {
                    return to_error_response("Access Point not exists.");
                }
                a.push(access_point);
                self.access_point_repo.store_access_points(a.clone());
                to_success_response(a)
            }
            None => to_error_response("Unable to find Account.")
        }
    }

    fn remove_access_point(&self, access_point: AccessPoint) -> HttpResponse<Vec<AccessPoint>> {
        match self.access_point_repo.get_access_points() {
            Some(content) => {
                let a: Vec<AccessPoint> = content.iter()
                    .filter(|x| x.pub_key != access_point.pub_key)
                    .cloned()
                    .collect();
                if a.len() == content.len() {
                    return to_error_response("Access Point not exists.");
                }
                self.access_point_repo.store_access_points(a.clone());
                to_success_response(a)
            }
            None => to_error_response("Unable to find Account.")
        }
    }
}

