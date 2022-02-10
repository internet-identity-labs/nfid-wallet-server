use crate::http::requests::AccountResponse;
use crate::{ConfigurationRepo, HttpResponse};
use crate::mapper::account_mapper::{account_request_to_account, account_to_account_response};
use crate::phone_number_service::PhoneNumberServiceTrait;
use crate::repository::account_repo::AccountRepoTrait;
use crate::requests::{HTTPAccountRequest, HTTPAccountUpdateRequest};
use crate::response_mapper::to_error_response;
use crate::response_mapper::to_success_response;
use crate::service::ic_service;
use crate::util::validation_util::validate_name;

pub trait AccountServiceTrait {
    fn get_account(&mut self) -> HttpResponse<AccountResponse>;
    fn create_account(&mut self, account_request: HTTPAccountRequest) -> HttpResponse<AccountResponse>;
    fn update_account(&mut self, account_request: HTTPAccountUpdateRequest) -> HttpResponse<AccountResponse>;
}

#[derive(Default)]
pub struct AccountService<T, N> {
    pub account_repo: T,
    pub phone_number_service: N,
}

impl<T: AccountRepoTrait, N: PhoneNumberServiceTrait> AccountServiceTrait for AccountService<T, N> {
    fn get_account(&mut self) -> HttpResponse<AccountResponse> {
        match self.account_repo.get_account() {
            Some(content) => to_success_response(account_to_account_response(content.clone())),
            None => to_error_response("Unable to find Account")
        }
    }

    fn create_account(&mut self, account_request: HTTPAccountRequest) -> HttpResponse<AccountResponse> {
        let princ = ic_service::get_caller().to_text();
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

        let is_whitelisted = ConfigurationRepo::get().whitelisted
            .contains(&account_request.phone_number);

        if !is_whitelisted && self.phone_number_service.is_exist(&phone_number_hash) {
            return to_error_response("Phone number already exists");
        }

        match self.phone_number_service.validate_token(&phone_number_hash, &token_hash) {
            Ok(_) => (),
            Err(message) => return to_error_response(message)
        };

        if !validate_name(account_request.name.clone().as_str()) {
            return to_error_response("Name must only contain letters and numbers (5-15 characters)");
        }

        let acc = account_request_to_account(account_request);
        match { self.account_repo.create_account(acc.clone()) } {
            None => {
                to_error_response("It's impossible to link this II anchor, please try another one.")
            }
            Some(_) => {
                self.phone_number_service.add(phone_number_hash);
                to_success_response(account_to_account_response(acc))
            }
        }
    }

    fn update_account(&mut self, account_request: HTTPAccountUpdateRequest) -> HttpResponse<AccountResponse> {
        match self.account_repo.get_account() {
            Some(acc) => {
                let mut new_acc = acc.clone();
                if !account_request.name.is_none() {
                    new_acc.name = account_request.name.unwrap();
                }
                self.account_repo.store_account(new_acc.clone());
                to_success_response(account_to_account_response(new_acc))
            }
            None => to_error_response("Unable to find Account.")
        }
    }
}




