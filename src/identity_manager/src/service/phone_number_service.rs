use blake3::Hash;
use ic_cdk::api::caller;
use ic_cdk::export::Principal;

use crate::ConfigurationRepo;
use crate::HTTPVerifyPhoneNumberRequest;
use crate::HttpResponse;
use crate::repo::{PhoneNumberRepo, TokenRepo};
use crate::response_mapper::{to_success_response, too_many_requests};
use crate::unauthorized;

pub fn validate_phone_number(phone_number: String) -> HttpResponse<bool> {
    if !is_lambda(&caller()) {
        return unauthorized();
    }

    if is_whitelisted(&phone_number) {
        return to_success_response(true)
    }

    let phone_number_hash = blake3::keyed_hash(
        &ConfigurationRepo::get().key,
        phone_number.as_bytes()
    );

    if is_too_many_requests(&phone_number_hash) {
        return too_many_requests()
    }

    let is_valid = !PhoneNumberRepo::is_exist(&phone_number_hash);
    to_success_response(is_valid)
}

pub fn post_token(request: HTTPVerifyPhoneNumberRequest) -> HttpResponse<bool> {
    if !ConfigurationRepo::get().lambda.eq(&caller()) {
        return unauthorized();
    }

    let phone_number_hash = blake3::keyed_hash(
        &ConfigurationRepo::get().key,
        request.phone_number.as_bytes()
    );

    let token_hash = blake3::keyed_hash(
        &ConfigurationRepo::get().key,
        request.token.as_bytes()
    );

    TokenRepo::add(phone_number_hash, token_hash);
    HttpResponse { status_code: 200, data: Some(true), error: None }
}

pub fn validate_token<'a>(phone_number_hash: &'a Hash, token_hash: &'a Hash) -> Result<(), &'a str> {
    return match TokenRepo::get(
        phone_number_hash,
        ConfigurationRepo::get().token_ttl
    ) {
        None => Err("Phone number not found"),
        Some(token) => {
            return match  token_hash.eq(token) {
                false => Err("Token does not match"),
                true => Ok(())
            }
        }
    }
}

fn is_whitelisted(phone_number: &String) -> bool {
    ConfigurationRepo::get().whitelisted.contains(phone_number)
}

fn is_lambda(caller: &Principal) -> bool {
    ConfigurationRepo::get().lambda.eq(caller)
}

fn is_too_many_requests(phone_number_hash: &Hash) -> bool {
    TokenRepo::get(
        phone_number_hash,
        ConfigurationRepo::get().token_refresh_ttl
    ).is_some()
}




