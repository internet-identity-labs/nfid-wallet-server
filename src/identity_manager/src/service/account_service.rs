use async_trait::async_trait;
use ic_cdk::{trap};
use itertools::Itertools;

use crate::http::response_mapper::ErrorResponse;
use crate::repository::repo::ConfigurationRepo;
use crate::service::http_outcall_service::{CanisterHttpRequestArgument, HttpMethod, HttpHeader, http_request, TransformContext, transorm_response_no_headers};
use crate::{AccessPointServiceTrait, Account, get_caller, HttpResponse};
use crate::http::requests::{AccountResponse, DeviceType, WalletVariant};
use crate::ic_service::{KeyType};
use crate::mapper::access_point_mapper::access_point_request_to_access_point;
use crate::mapper::account_mapper::{account_request_to_account, account_to_account_response};
use crate::repository::account_repo::AccountRepoTrait;
use crate::requests::{AccountRequest, AccountUpdateRequest};
use crate::response_mapper::to_error_response;
use crate::response_mapper::to_success_response;
use crate::service::ic_service;
use crate::service::ic_service::{DeviceData};
use crate::service::security_service::secure_2fa;
use crate::util::validation_util::validate_name;
use serde::{Deserialize, Serialize};

use super::email_validation_service;

#[async_trait(? Send)]
pub trait AccountServiceTrait {
    fn get_account_response(&mut self) -> HttpResponse<AccountResponse>;
    fn get_account(&mut self) -> Option<Account>;
    fn update_2fa(&mut self, state: bool) -> AccountResponse;
    async fn create_account(&mut self, account_request: AccountRequest) -> HttpResponse<AccountResponse>;
    async fn update_account(&mut self, account_request: AccountUpdateRequest) -> HttpResponse<AccountResponse>;
    fn remove_account(&mut self) -> HttpResponse<bool>;
    fn remove_account_by_principal(&mut self, principal: String) -> HttpResponse<bool>;
    fn store_accounts(&mut self, accounts: Vec<Account>) -> HttpResponse<bool>;
    fn get_account_by_anchor(&mut self, anchor: u64, wallet: WalletVariant) -> HttpResponse<AccountResponse>;
    fn get_account_by_principal(&mut self, princ: String) -> HttpResponse<AccountResponse>;
    fn get_root_id_by_principal(&mut self, princ: String) -> Option<String>;
    async fn recover_account(&mut self, anchor: u64, wallet: Option<WalletVariant>) -> HttpResponse<AccountResponse>;
    fn get_all_accounts(&mut self) -> Vec<Account>;
    async fn validate_email_and_principal(email: &str, principal: &str) -> bool;
}

#[derive(Default)]
pub struct AccountService<T, A> {
    pub account_repo: T,
    pub access_point_service: A,
}

#[async_trait(? Send)]
impl<T: AccountRepoTrait, A: AccessPointServiceTrait> AccountServiceTrait for AccountService<T, A> {
    fn get_account_response(&mut self) -> HttpResponse<AccountResponse> {
        match self.account_repo.get_account() {
            Some(content) => to_success_response(account_to_account_response(content.clone())),
            None => to_error_response("Unable to find Account")
        }
    }

    fn get_account(&mut self) -> Option<Account> {
        self.account_repo.get_account()
    }

    fn update_2fa(&mut self, state: bool) -> AccountResponse {
        match self.account_repo.get_account() {
            None => {
                trap("No such Account")
            }
            Some(mut acc) => {
                if !acc.access_points.clone().into_iter()
                    .any(|l| l.device_type.eq(&DeviceType::Passkey)) {
                    trap("Forbidden")
                }
                acc.is2fa_enabled = state;
                self.account_repo.store_account(acc.clone());
                account_to_account_response(acc)
            }
        }
    }

    async fn create_account(&mut self, account_request: AccountRequest) -> HttpResponse<AccountResponse> {
        let princ = ic_service::get_caller().to_text();
        if ic_service::is_anonymous(princ.clone()) {
            return to_error_response("User is anonymous");
        }
        let mut devices: Vec<DeviceData> = Vec::default();
        let mut acc = account_request_to_account(account_request.clone());
        if account_request.email.is_some() {
            if !email_validation_service::contains(account_request.email.clone().unwrap().to_string(), princ) {
                trap("Email and principal are not valid.")
            }
        }
        if acc.wallet.eq(&WalletVariant::NFID) {
            if account_request.email.is_none() {
                trap("Email is empty");
            }
            let anchor = self.account_repo.find_next_nfid_anchor();
            acc.anchor = anchor;
            match account_request.access_point {
                None => {
                    trap("Device Data required")
                }
                Some(dd) => {
                    if !acc.principal_id.eq(&dd.pub_key) {
                        trap("Incorrect Device Data")
                    }
                    if !&dd.device_type.eq(&DeviceType::Email) {
                        trap("Only email device can be registered as a root")
                    }
                    acc.access_points.insert(access_point_request_to_access_point(dd));
                }
            }
        } else {
            devices = ic_service::trap_if_not_authenticated(acc.anchor.clone(), get_caller()).await;
        }
        match { self.account_repo.create_account(acc.clone()) } {
            None => {
                to_error_response("Impossible to link this II anchor, please try another one.")
            }
            Some(_) => {
                let recovery_device = devices.into_iter()
                    .find(|dd| dd.key_type.eq(&KeyType::SeedPhrase));
                match recovery_device {
                    None => {}
                    Some(rd) => {
                        acc = self.access_point_service.migrate_recovery_device(rd, &acc);
                    }
                }
                to_success_response(account_to_account_response(acc))
            }
        }
    }

    async fn update_account(&mut self, account_request: AccountUpdateRequest) -> HttpResponse<AccountResponse> {
        match self.account_repo.get_account() {
            Some(acc) => {
                let mut new_acc = acc.clone();
                if !&account_request.name.is_none() {
                    if !validate_name(account_request.name.as_ref().unwrap().as_str()) {
                        return to_error_response("Name must only contain letters and numbers (5-15 characters)");
                    }
                    new_acc.name = account_request.name.clone();
                }
                if account_request.email.is_some() {
                    if new_acc.email.as_ref().is_some() {
                        return HttpResponse::error(400, "Email cannot be updated if set once.")
                    }
                    let princ = ic_service::get_caller().to_text();
                    let email = account_request.email.clone().unwrap();
                    if !Self::validate_email_and_principal(&email, &princ).await {
                        trap("Email and principal are not valid.")
                    }
                    new_acc.email = account_request.email;
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

        to_success_response(true)
    }

    //remove with PN
    fn remove_account_by_principal(&mut self, principal: String) -> HttpResponse<bool> {
        let result = self.account_repo.remove_account_by_principal(principal);
        if result.is_none() {
            return to_error_response("Unable to remove Account");
        }

        to_success_response(true)
    }

    fn store_accounts(&mut self, accounts: Vec<Account>) -> HttpResponse<bool> {
        self.account_repo.store_accounts(accounts);
        to_success_response(true)
    }

    fn get_account_by_anchor(&mut self, anchor: u64, wallet: WalletVariant) -> HttpResponse<AccountResponse> {
        match { self.account_repo.get_account_by_anchor(anchor, wallet) } {
            None => {
                to_error_response("Anchor not registered.")
            }
            Some(acc) => {
                to_success_response(account_to_account_response(acc))
            }
        }
    }

    fn get_account_by_principal(&mut self, princ: String) -> HttpResponse<AccountResponse> {
        match { self.account_repo.get_account_by_principal(princ) } {
            None => {
                to_error_response("Principal not registered.")
            }
            Some(acc) => {
                to_success_response(account_to_account_response(acc))
            }
        }
    }

    fn get_root_id_by_principal(&mut self, princ: String) -> Option<String> {
        match { self.account_repo.get_account_by_principal(princ) } {
            None => {
                None
            }
            Some(acc) => {
                Some(acc.principal_id)
            }
        }
    }


    async fn recover_account(&mut self, anchor: u64, wallet: Option<WalletVariant>) -> HttpResponse<AccountResponse> {
        let vw = match wallet.clone() {
            None => { WalletVariant::InternetIdentity }
            Some(x) => { x }
        };
        if vw.eq(&WalletVariant::InternetIdentity) {
            match { self.account_repo.get_account_by_anchor(anchor, vw) } {
                None => {
                    let account = AccountRequest { anchor, wallet, access_point: None, email: None };
                    self.create_account(account).await
                }
                Some(acc) => {
                    //TODO looks like we can recover not only with the recovery phrase but with every registered device (bug?)
                    ic_service::trap_if_not_authenticated(anchor.clone(), get_caller()).await;
                    to_success_response(account_to_account_response(acc))
                }
            }
        } else {
            secure_2fa();
            match { self.account_repo.get_account() } {
                None => {
                    trap("Recovery not registered")
                }
                Some(account) => {
                    if !account.anchor.eq(&anchor) {
                        trap("Recovery not registered")
                    }
                    to_success_response(account_to_account_response(account))
                }
            }
        }
    }

    fn get_all_accounts(&mut self) -> Vec<Account> {
        self.account_repo.get_all_accounts()
    }

    async fn validate_email_and_principal(email: &str, principal: &str) -> bool {
        let url = &ConfigurationRepo::get().lambda_url;
    
        #[derive(Debug, Deserialize, Serialize)]
        struct CheckEmailAndPrincipalExistsResponse {
            isExists: bool,
        }
    
        let request = CanisterHttpRequestArgument {
            url: url.clone() + "/check_email_and_principal_exists",
            method: HttpMethod::POST,
            headers: vec![
                HttpHeader {
                    name: "Content-Type".to_string(),
                    value: "application/json".to_string(),
                },
            ],
            body: Some(format!(r#"{{"email": "{}", "principal": "{}"}}"#, email, principal).into_bytes()),
            max_response_bytes: Some(700),
            transform: Some(TransformContext::new(transorm_response_no_headers, vec![]))
        };
    
        match http_request(request).await {
            Ok((response,)) => {
                let str_body = String::from_utf8(response.body).expect("Not UTF-8 encoded.");
                let json: CheckEmailAndPrincipalExistsResponse = serde_json::from_str(&str_body).expect("JSON parsing failed");
                json.isExists
            }
            Err((r, m)) => {
                let message = format!("RejectionCode: {:#?} and message: {}.", r, m);
                trap(&message);
            }
        }
    }
}