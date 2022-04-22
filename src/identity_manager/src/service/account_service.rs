use crate::http::requests::AccountResponse;
use crate::{Account, HttpResponse};
use crate::mapper::account_mapper::{account_request_to_account, account_to_account_response};

use crate::repository::account_repo::AccountRepoTrait;
use crate::repository::phone_number_repo::PhoneNumberRepoTrait;
use crate::requests::{AccountRequest, AccountUpdateRequest};
use crate::response_mapper::to_error_response;
use crate::response_mapper::to_success_response;
use crate::service::ic_service;
use crate::util::validation_util::validate_name;


pub trait AccountServiceTrait {
    fn get_account(&mut self) -> HttpResponse<AccountResponse>;
    fn create_account(&mut self, account_request: AccountRequest) -> HttpResponse<AccountResponse>;
    fn update_account(&mut self, account_request: AccountUpdateRequest) -> HttpResponse<AccountResponse>;
    fn remove_account(&mut self) -> HttpResponse<bool>;
    fn store_accounts(&mut self, accounts: Vec<Account>) -> HttpResponse<bool>;
    fn get_phone_number_sha2(&self, principal_id: String) -> HttpResponse<String> ;
}

#[derive(Default)]
pub struct AccountService<T, N> {
    pub account_repo: T,
    pub phone_number_repo: N,
}

impl<T: AccountRepoTrait, N: PhoneNumberRepoTrait> AccountServiceTrait for AccountService<T, N> {
    fn get_account(&mut self) -> HttpResponse<AccountResponse> {
        match self.account_repo.get_account() {
            Some(content) => to_success_response(account_to_account_response(content.clone())),
            None => to_error_response("Unable to find Account")
        }
    }

    fn create_account(&mut self, account_request: AccountRequest) -> HttpResponse<AccountResponse> {
        let princ = ic_service::get_caller().to_text();
        if ic_service::is_anonymous(princ) {
            return to_error_response("User is anonymous");
        }

        let acc = account_request_to_account(account_request);
        match { self.account_repo.create_account(acc.clone()) } {
            None => {
                to_error_response("It's impossible to link this II anchor, please try another one.")
            }
            Some(_) => {
                to_success_response(account_to_account_response(acc))
            }
        }
    }

    fn update_account(&mut self, account_request: AccountUpdateRequest) -> HttpResponse<AccountResponse> {
        match self.account_repo.get_account() {
            Some(acc) => {
                let mut new_acc = acc.clone();
                if !&account_request.name.is_none() {
                    if !validate_name(account_request.name.as_ref().unwrap().as_str()) {
                        return to_error_response("Name must only contain letters and numbers (5-15 characters)");
                    }
                    new_acc.name = account_request.name.clone();
                }
                new_acc.base_fields.update_modified_date();
                self.account_repo.store_account(new_acc.clone());
                to_success_response(account_to_account_response(new_acc))
            }
            None => to_error_response("Unable to find Account.")
        }
    }

    fn remove_account(&mut self) -> HttpResponse<bool> {
        let result = self.account_repo.remove_account();
        if result.is_none() {
            return to_error_response("Unable to remove Account");
        }

        let account = result.unwrap();
        if account.phone_number.is_none() {
            return to_success_response(true);
        }

        let phone_number = account.phone_number.unwrap();
        let success = self.phone_number_repo.remove(&phone_number);

        if !success {
            return to_error_response("Unable to remove Phone Number");
        }

        to_success_response(true)
    }

    fn store_accounts(&mut self, accounts: Vec<Account>) -> HttpResponse<bool> {
        self.account_repo.store_accounts(accounts);
        to_success_response(true)
    }

    fn get_phone_number_sha2(&self, principal_id: String) -> HttpResponse<String> {
        match self.account_repo.get_account_by_id(principal_id)
        {
            None => { to_error_response("Account not exist") }
            Some(account) => {
                match account.phone_number_sha2 {
                    None => { to_error_response("Phone number not verified") }
                    Some(pn_sha2) => { to_success_response(pn_sha2) }
                }
            }
        }
    }
}




