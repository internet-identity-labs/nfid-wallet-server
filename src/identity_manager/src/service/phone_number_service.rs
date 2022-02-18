use ic_cdk::export::Principal;
use crate::{AccountRepo, ConfigurationRepo, ic_service, Response, TokenRequest, ValidatePhoneRequest};
use crate::HttpResponse;
use crate::repository::account_repo::AccountRepoTrait;
use crate::repository::encrypt::account_encrypt::encrypt;
use crate::repository::phone_number_repo::PhoneNumberRepoTrait;
use crate::response_mapper::{error_response, response, to_error_response, to_success_response, too_many_requests};
use crate::repository::token_repo::{TokenRepoTrait};

pub trait PhoneNumberServiceTrait {
    fn validate_phone(&self, request: ValidatePhoneRequest) -> Response;
    fn post_token(&self, request: TokenRequest) -> Response;
    fn verify_token(&self, token: String) -> Response;
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
            return error_response(400, "Incorrect format of principal.")
        }

        if !self.account_repo.exists(&result.unwrap()) {
            return error_response(404, "Account not found.");
        }

        let principal_id_encrypted = encrypt(request.principal_id);
        let ttl = ConfigurationRepo::get().token_refresh_ttl;
        if self.token_repo.get(&principal_id_encrypted, ttl).is_some() {
            return error_response(429, "Too many requests.");
        }

        let phone_number_encrypted = encrypt(request.phone_number.clone());
        if self.phone_number_repo.is_exist(&phone_number_encrypted) {
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
            return error_response(400, "Incorrect format of principal.")
        }

        let principal = result.unwrap();
        if !self.account_repo.exists(&principal) {
            return error_response(404, "Account not found.");
        }

        let principal_id_encrypted = encrypt(principal.to_text());
        let token_encrypted = encrypt(request.token);
        let phone_number_encrypted = encrypt(request.phone_number);
        self.token_repo.add(principal_id_encrypted, token_encrypted, phone_number_encrypted);

        response(200)
    }

    fn verify_token(&self, token: String) -> Response {
        let account_opt = self.account_repo.get_account();
        if account_opt.is_none() {
            return error_response(404, "Account not found.");
        }

        let principal_id_encrypted = encrypt(ic_service::get_caller().to_text());
        let ttl = ConfigurationRepo::get().token_ttl;
        let value_opt = self.token_repo.get(&principal_id_encrypted, ttl);
        if value_opt.is_none() {
            return error_response(404, "Principal id not found.");
        }

        let token_encrypted = encrypt(token);
        let (token_encrypted_persisted, phone_number_encrypted_persisted) = value_opt.unwrap();
        if !token_encrypted.eq(token_encrypted_persisted) {
            return error_response(400, "Token does not match.");
        }

        let mut account = account_opt.unwrap();
        account.phone_number = Some(phone_number_encrypted_persisted.clone());
        self.account_repo.store_account(account);
        self.phone_number_repo.add(phone_number_encrypted_persisted.clone());

        response(200)
    }
}




