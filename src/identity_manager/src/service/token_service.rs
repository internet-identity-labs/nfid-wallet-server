use crate::{CONFIGURATION, HTTPAccountRequest, HttpResponse, HTTPVerifyPhoneNumberRequest,
            TOKEN_STORAGE, unauthorized};

pub fn post_token(request: HTTPVerifyPhoneNumberRequest) -> HttpResponse<bool> {
    let principal = &ic_cdk::api::caller().to_text();

    if !is_lambda_authenticated(principal) {
        return unauthorized();
    }

    let phone_number_hash = blake3::hash(request.phone_number.as_bytes());
    let token_hash = blake3::hash(request.token.as_bytes());

    TOKEN_STORAGE.with(|storage| {
        storage.borrow_mut().insert(phone_number_hash, token_hash);
        HttpResponse { status_code: 200, data: Some(true), error: None }
    })
}

pub fn validate_token(request: &HTTPAccountRequest) -> Result<(), &str> {
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
        };
    })
}

fn is_lambda_authenticated(principal: &String) -> bool {
    CONFIGURATION.with(|option| {
        option.borrow()
            .clone()
            .map(|x| x.lambda)
            .map(|x| x.to_text())
            .filter(|x| principal.eq(x))
            .is_some()
    })
}




