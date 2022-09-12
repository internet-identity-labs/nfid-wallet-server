use ic_cdk::export::Principal;

use crate::{ConfigurationRepo, ic_service, Response, TokenRequest, ValidatePhoneRequest};
use crate::repository::account_repo::AccountRepoTrait;
use crate::repository::phone_number_repo::PhoneNumberRepoTrait;
use crate::repository::token_repo::TokenRepoTrait;
use crate::response_mapper::{error_response, response};

pub trait PhoneNumberServiceTrait {
    fn validate_phone(&self, request: ValidatePhoneRequest) -> Response;
    fn post_token(&self, request: TokenRequest) -> Response;
    fn verify_token(&self, token: String) -> Response;
    fn detach_test_phone_number(&self) -> ();
}

#[derive(Default)]
pub struct PhoneNumberService<T, N, A> {
    pub(crate) phone_number_repo: T,
    pub(crate) token_repo: N,
    pub(crate) account_repo: A,
}

impl<T: PhoneNumberRepoTrait, N: TokenRepoTrait, A: AccountRepoTrait> PhoneNumberServiceTrait for PhoneNumberService<T, N, A> {
    fn validate_phone(&self, request: ValidatePhoneRequest) -> Response {
        if !ConfigurationRepo::get().lambda.eq(&ic_service::get_caller()) {
            return error_response(403, "Unauthorized.");
        }

        if request.principal_id.len() < 10 {
            return error_response(403, "Anonymous user is forbidden.");
        }

        let result = Principal::from_text(&request.principal_id);
        if result.is_err() {
            return error_response(400, "Incorrect format of principal.");
        }

        if !self.account_repo.exists(&result.unwrap()) {
            return error_response(404, "Account not found.");
        }

        let ttl = ConfigurationRepo::get().token_refresh_ttl;
        if self.token_repo.get(&request.principal_id, ttl).is_some() {
            return error_response(429, "Too many requests.");
        }

        if self.phone_number_repo.is_exist(&request.phone_number_hash.clone()) {
            return response(204);
        }

        response(200)
    }

    fn post_token(&self, request: TokenRequest) -> Response {
        if !ConfigurationRepo::get().lambda.eq(&ic_service::get_caller()) {
            return error_response(403, "Unauthorized.");
        }

        if request.principal_id.len() < 10 {
            return error_response(403, "Anonymous user is forbidden.");
        }

        let result = Principal::from_text(request.principal_id);
        if result.is_err() {
            return error_response(400, "Incorrect format of principal.");
        }

        let principal = result.unwrap();
        if !self.account_repo.exists(&principal) {
            return error_response(404, "Account not found.");
        }

        self.token_repo.add(principal.to_text(), request.token, request.phone_number_encrypted, request.phone_number_hash);

        response(200)
    }

    fn verify_token(&self, token: String) -> Response {
        let account_opt = self.account_repo.get_account();
        if account_opt.is_none() {
            return error_response(404, "Account not found.");
        }

        let ttl = ConfigurationRepo::get().token_ttl;
        let mut account = account_opt.unwrap();
        let value_opt = self.token_repo.get(&account.principal_id, ttl);
        if value_opt.is_none() {
            return error_response(404, "Principal id not found.");
        }

        let (token_persisted, phone_number_persisted, phone_number_sha2) = value_opt.unwrap();
        if !token.eq(token_persisted) {
            return error_response(400, "Incorrect verification code, please try again.");
        }
        account.phone_number = Some(phone_number_persisted.clone());
        account.phone_number_sha2 = Some(phone_number_sha2.clone());
        self.account_repo.store_account(account);
        self.phone_number_repo.add(phone_number_sha2.clone());

        response(200)
    }

    fn detach_test_phone_number(&self) -> () {
        self.phone_number_repo.remove(&String::from("8fba797bcc5427ca466bf5ef0d8fcc69636fa6b67ea93e240198ecaac3df3716"));
    }
}




