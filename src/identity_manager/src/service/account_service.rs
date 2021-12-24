use crate::{HttpResponse, TOKEN_STORAGE};
use crate::service::principle_service::get_principal;
use crate::repository::repo::{Account, AccountRepo, Device, Persona};
use crate::requests::{HTTPAccountRequest, HTTPAccountUpdateRequest};
use crate::response_mapper::to_error_response;
use crate::response_mapper::to_success_response;

pub fn get_account() -> HttpResponse<Account> {
    let principal_id = &ic_cdk::api::caller().to_text();
    match AccountRepo::get_account(get_principal(principal_id)) {
        Some(content) => to_success_response(content.clone()),
        None => to_error_response("Unable to find Account")
    }
}

pub fn create_account(account_request: HTTPAccountRequest) -> HttpResponse<Account> {
    match validate_token(&account_request)  {
        Ok(_) => (),
        Err(message) => return to_error_response(message)
    };

    let princ = &ic_cdk::api::caller().to_text();
    let devices: Vec<Device> = Vec::new();
    let personas: Vec<Persona> = Vec::new();
    let acc = Account {
        principal_id: princ.clone(),
        name: account_request.name,
        phone_number: account_request.phone_number,
        email: account_request.email,
        devices,
        personas,
    };
    AccountRepo::store_account(princ.clone(), acc.clone());
    to_success_response(acc)
}

pub fn update_account(account_request: HTTPAccountUpdateRequest) -> HttpResponse<Account> {
    let p = &ic_cdk::api::caller().to_text();
    match AccountRepo::get_account(get_principal(p)) {
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
            AccountRepo::store_account(new_acc.principal_id.clone(), new_acc.clone());
            to_success_response(new_acc)
        }
        None => to_error_response("Unable to find Account.")
    }
}

fn validate_token(request: &HTTPAccountRequest) -> Result<(), &str> {
    let phone_number_hash = blake3::hash(request.phone_number.as_bytes());
    let token_hash = blake3::hash(request.token.as_bytes());

    TOKEN_STORAGE.with(|storage| {
        return match storage.borrow().get(&phone_number_hash) {
            Some(token) => {
                return match token_hash.eq(token) {
                    true => Ok(()),
                    false => Err("Token does not match")
                };
            }
            None => Err("Phone number not found")
        }
    })
}




