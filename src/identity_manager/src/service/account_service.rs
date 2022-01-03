use crate::{HttpResponse, token_service};
use crate::http::requests::AccountRR;
use crate::mapper::account_mapper::account_to_account_response;
use crate::repository::repo::{Account, AccountRepo, calculate_hash, Device, Persona};
use crate::requests::{HTTPAccountRequest, HTTPAccountUpdateRequest};
use crate::response_mapper::to_error_response;
use crate::response_mapper::to_success_response;

pub fn get_account() -> HttpResponse<AccountRR> {
    match AccountRepo::get_account() {
        Some(content) => to_success_response(account_to_account_response(content.clone())),
        None => to_error_response("Unable to find Account")
    }
}

pub fn create_account(account_request: HTTPAccountRequest) -> HttpResponse<AccountRR> {
    match token_service::validate_token(&account_request) {
        Ok(_) => (),
        Err(message) => return to_error_response(message)
    };

    let princ = &ic_cdk::api::caller().to_text();
    let devices: Vec<Device> = Vec::new();
    let personas: Vec<Persona> = Vec::new();
    let hashed_id = calculate_hash(princ);
    let acc = Account {
        principal_id_hash: hashed_id,
        principal_id: princ.clone(),
        name: account_request.name.clone(),
        phone_number: account_request.phone_number.clone(),
        email: account_request.email.clone(),
        devices,
        personas,
    };
    AccountRepo::store_account(acc.clone());
    to_success_response(account_to_account_response(acc))
}

pub fn update_account(account_request: HTTPAccountUpdateRequest) -> HttpResponse<AccountRR> {
    match AccountRepo::get_account() {
        Some(acc) => {
            let mut new_acc = acc.clone();
            if !account_request.email.is_none() {
                new_acc.email = account_request.email.unwrap();
            }
            if !account_request.phone_number.is_none() {
                new_acc.phone_number = account_request.phone_number.unwrap();
            }
            if !account_request.name.is_none() {
                new_acc.name = account_request.name.unwrap();
            }
            AccountRepo::store_account(new_acc.clone());
            to_success_response(account_to_account_response(new_acc))
        }
        None => to_error_response("Unable to find Account.")
    }
}




