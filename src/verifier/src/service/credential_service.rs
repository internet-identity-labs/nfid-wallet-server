use std::convert::TryInto;

use ic_cdk::{call, trap};
use ic_cdk::export::Principal;

use crate::{Credential, resolve_certificate, TokenKey, verify_phone_number_existence};
use crate::http::responses::{HttpResponse, to_success_response};
use crate::repository::credential_repo::get_credential;
use crate::repository::token_repo;
use crate::repository::token_repo::{generate_token, Token};

pub async fn generate_pn_token(domain: String) -> TokenKey {
    let principal = ic_cdk::api::caller().to_text();

    let res: Vec<u8> = match call(Principal::management_canister(), "raw_rand", ()).await {
        Ok((res, )) => res,
        Err((_, err)) => trap(&format!("failed to get salt: {}", err)),
    };

    let token_key: TokenKey = res[..].try_into().unwrap_or_else(|_| {
        trap(&format!(
            "Expected raw randomness to be of length 32, got {}",
            res.len()
        ));
    });

    generate_token(principal, domain, token_key.clone())
}

pub fn is_phone_number_approved(who: String) -> HttpResponse<bool> {
    return match get_credential(who) {
        Some(credential) => {
            let resp = credential.phone_number_sha2.is_some();
            to_success_response(resp)
        }
        None => {
            to_success_response(false)
        }
    };
}

pub async fn resolve_token(token_key: TokenKey) -> Option<Credential> {
    let principal = ic_cdk::api::caller().to_text();

    let domain = token_repo::resolve_token(token_key);
    let phone_number_sha2 = verify_phone_number_existence(principal, domain).await;
    resolve_certificate(phone_number_sha2, token_key)
}