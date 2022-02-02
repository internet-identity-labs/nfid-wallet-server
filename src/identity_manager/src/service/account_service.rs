use crate::http::requests::AccountResponse;
use crate::{ConfigurationRepo, HttpResponse, phone_number_service};
use crate::mapper::account_mapper::{account_request_to_account, account_to_account_response};
use crate::repo::PhoneNumberRepo;
use crate::repository::repo::{Account, AccountRepo, AccessPoint, Persona};
use crate::requests::{HTTPAccountRequest, HTTPAccountUpdateRequest};
use crate::response_mapper::to_error_response;
use crate::response_mapper::to_success_response;
use crate::util::validation_util::validate_name;

pub fn get_account() -> HttpResponse<AccountResponse> {
    match AccountRepo::get_account() {
        Some(content) => to_success_response(account_to_account_response(content.clone())),
        None => to_error_response("Unable to find Account")
    }
}

pub fn create_account(account_request: HTTPAccountRequest) -> HttpResponse<AccountResponse> {
    let princ = &ic_cdk::api::caller().to_text();
    if princ.len() < 10 {
        return to_error_response("User is anonymous");
    }
    let phone_number_hash = blake3::keyed_hash(
        &ConfigurationRepo::get().key,
        account_request.phone_number.as_bytes(),
    );
    let token_hash = blake3::keyed_hash(
        &ConfigurationRepo::get().key,
        account_request.token.as_bytes(),
    );

    if PhoneNumberRepo::is_exist(&phone_number_hash) {
        return to_error_response("Phone number already exists");
    }

    match phone_number_service::validate_token(&phone_number_hash, &token_hash) {
        Ok(_) => (),
        Err(message) => return to_error_response(message)
    };

    if !validate_name(account_request.name.clone().as_str()) {
        return to_error_response("Name must only contain letters and numbers (5-15 characters)");
    }

    let acc = account_request_to_account(account_request);
    match { AccountRepo::create_account(acc.clone()) } {
        None => {
            to_error_response("It's impossible to link this II anchor, please try another one.") }
        Some(_) => {
            PhoneNumberRepo::add(phone_number_hash);
            to_success_response(account_to_account_response(acc))
        }
    }
}

pub fn update_account(account_request: HTTPAccountUpdateRequest) -> HttpResponse<AccountResponse> {
    match AccountRepo::get_account() {
        Some(acc) => {
            let mut new_acc = acc.clone();
            if !account_request.name.is_none() {
                new_acc.name = account_request.name.unwrap();
            }
            AccountRepo::store_account(new_acc.clone());
            to_success_response(account_to_account_response(new_acc))
        }
        None => to_error_response("Unable to find Account.")
    }
}




