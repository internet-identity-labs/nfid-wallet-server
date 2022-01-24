use blake3::Hash;
use ic_cdk::api::caller;
use crate::{ConfigurationRepo, TOKEN_STORAGE};
use crate::HTTPVerifyPhoneNumberRequest;
use crate::HttpResponse;
use crate::unauthorized;

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

    TOKEN_STORAGE.with(|storage| {
        storage.borrow_mut().insert(phone_number_hash, token_hash);
        HttpResponse { status_code: 200, data: Some(true), error: None }
    })
}

pub fn validate_token<'a>(phone_number_hash: &'a Hash, token_hash: &'a Hash) -> Result<(), &'a str> {
    TOKEN_STORAGE.with(|storage| {
        return match storage.borrow_mut().get(&phone_number_hash) {
            Some(token) => {
                return match token_hash.eq(token) {
                    true => Ok(()),
                    false => Err("Token does not match")
                };
            }
            None => Err("Phone number not found")
        };
    })
}




