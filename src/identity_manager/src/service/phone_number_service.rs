use blake3::Hash;
use ic_cdk::export::Principal;

use crate::{ConfigurationRepo, ic_service};
use crate::HttpResponse;
use crate::HTTPVerifyPhoneNumberRequest;
use crate::repository::phone_number_repo::PhoneNumberRepoTrait;
use crate::response_mapper::{to_success_response, too_many_requests};
use crate::unauthorized;
use crate::repository::token_repo::{TokenRepoTrait};

pub trait PhoneNumberServiceTrait {
    fn add(&self, phone_number_hash: blake3::Hash) -> ();
    fn is_exist(&self, phone_number_hash: &blake3::Hash) -> bool;
    fn validate_phone_number(&self, phone_number: String) -> HttpResponse<bool>;
    fn post_token(&self, request: HTTPVerifyPhoneNumberRequest) -> HttpResponse<bool>;
    fn validate_token(&self, phone_number_hash: &Hash, token_hash: &Hash) -> Result<(), &str>;
}

#[derive(Default)]
pub struct PhoneNumberService<T, N> {
    pub(crate) phone_number_repo: T,
    pub(crate) token_repo: N,
}

impl<T: PhoneNumberRepoTrait, N: TokenRepoTrait> PhoneNumberServiceTrait for PhoneNumberService<T, N> {
    fn add(&self, phone_number_hash: blake3::Hash) -> () {
        self.phone_number_repo.add(phone_number_hash)
    }

    fn is_exist(&self, phone_number_hash: &blake3::Hash) -> bool {
        self.phone_number_repo.is_exist(phone_number_hash)
    }

    fn validate_phone_number(&self, phone_number: String) -> HttpResponse<bool> {
        if !is_lambda(&ic_service::get_caller()) {
            return unauthorized();
        }

        if is_whitelisted(&phone_number) {
            return to_success_response(true);
        }

        let phone_number_hash = blake3::keyed_hash(
            &ConfigurationRepo::get().key,
            phone_number.as_bytes(),
        );

        if self.token_repo.get(
            &phone_number_hash,
            ConfigurationRepo::get().token_refresh_ttl,
        ).is_some() {
            return too_many_requests();
        }

        let is_valid = !self.phone_number_repo.is_exist(&phone_number_hash);
        to_success_response(is_valid)
    }

    fn post_token(&self, request: HTTPVerifyPhoneNumberRequest) -> HttpResponse<bool> {
        if !ConfigurationRepo::get().lambda.eq(&ic_service::get_caller()) {
            return unauthorized();
        }

        let phone_number_hash = blake3::keyed_hash(
            &ConfigurationRepo::get().key,
            request.phone_number.as_bytes(),
        );

        let token_hash = blake3::keyed_hash(
            &ConfigurationRepo::get().key,
            request.token.as_bytes(),
        );

        self.token_repo.add(phone_number_hash, token_hash);
        HttpResponse { status_code: 200, data: Some(true), error: None }
    }

    fn validate_token(&self, phone_number_hash: &Hash, token_hash: &Hash) -> Result<(), &str> {
        return match self.token_repo.get(
            phone_number_hash,
            ConfigurationRepo::get().token_ttl,
        ) {
            None => Err("Phone number not found"),
            Some(token) => {
                return match token_hash.eq(token) {
                    false => Err("Token does not match"),
                    true => Ok(())
                };
            }
        };
    }
}

fn is_whitelisted(phone_number: &String) -> bool {
    ConfigurationRepo::get().whitelisted.contains(phone_number)
}

fn is_lambda(caller: &Principal) -> bool {
    ConfigurationRepo::get().lambda.eq(caller)
}




