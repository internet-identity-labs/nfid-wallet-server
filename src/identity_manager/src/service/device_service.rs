use std::borrow::Borrow;
use crate::repository::repo::{Device, DeviceRepo};
use crate::response_mapper::{HttpResponse, to_error_response, to_success_response};

pub fn read_devices() -> HttpResponse<Vec<Device>> {
    match DeviceRepo::get_devices() {
        Some(content) => { to_success_response(content) }
        None => to_error_response("Unable to find Account.")
    }
}


pub fn create_device(device: Device) -> HttpResponse<bool> {
    match DeviceRepo::get_devices() {
        Some(mut content) => {
            if content.contains(device.borrow()) {
               return to_error_response("Device exists.");
            }
            content.push(device);
            DeviceRepo::store_devices(content);
            to_success_response(true)
        }
        None => to_error_response("Unable to find Account.")
    }
}

