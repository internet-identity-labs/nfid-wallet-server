use crate::service::principle_service::get_principal;
use crate::repository::repo::{Device, DeviceRepo};
use crate::response_mapper::{HttpResponse, to_error_response, to_success_response};

pub fn read_devices() -> HttpResponse<Vec<Device>> {
    let p = &ic_cdk::api::caller().to_text();
    match DeviceRepo::get_devices(get_principal(p)) {
        Some(content) => { to_success_response(content) }
        None => to_error_response("Unable to find Account.")
    }
}


pub fn create_device(device: Device) -> HttpResponse<bool> {
    let princ = &ic_cdk::api::caller().to_text();
    match DeviceRepo::get_devices(get_principal(princ)) {
        Some(mut content) => {
            content.push(device);
            DeviceRepo::store_devices(princ, content);
            to_success_response(true)
        }
        None => to_error_response("Unable to find Account.")
    }
}

