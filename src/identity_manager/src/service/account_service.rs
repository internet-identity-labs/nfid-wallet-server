use async_trait::async_trait;
use ic_cdk::trap;

use crate::http::requests::{AccountResponse, DeviceType, WalletVariant};
use crate::ic_service::KeyType;
use crate::mapper::access_point_mapper::access_point_request_to_access_point;
use crate::mapper::account_mapper::{account_request_to_account, account_to_account_response};
use crate::repository::account_repo::AccountRepoTrait;
use crate::requests::AccountRequest;
use crate::response_mapper::to_error_response;
use crate::response_mapper::to_success_response;
use crate::service::ic_service;
use crate::service::ic_service::DeviceData;
use crate::{get_caller, AccessPointServiceTrait, Account, HttpResponse};

use super::email_validation_service;

#[async_trait(? Send)]
pub trait AccountServiceTrait {
    fn get_account_response(&mut self) -> HttpResponse<AccountResponse>;
    fn get_account(&mut self) -> Option<Account>;
    fn update_2fa(&mut self, state: bool) -> AccountResponse;
    async fn create_account(
        &mut self,
        account_request: AccountRequest,
    ) -> HttpResponse<AccountResponse>;
    fn remove_account(&mut self) -> HttpResponse<bool>;
    fn get_account_by_anchor(
        &mut self,
        anchor: u64,
        wallet: WalletVariant,
    ) -> HttpResponse<AccountResponse>;
    fn get_account_by_principal(&mut self, princ: String) -> HttpResponse<AccountResponse>;
    fn get_root_id_by_principal(&mut self, princ: String) -> Option<String>;
    fn get_anchor_by_principal(&mut self, princ: String) -> Option<u64>;
    fn get_all_accounts(&mut self) -> Vec<Account>;
    async fn sync_recovery_phrase_from_internet_identity(
        &self,
        anchor: u64,
    ) -> HttpResponse<AccountResponse>;
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
            None => to_error_response("Unable to find Account"),
        }
    }

    fn get_account(&mut self) -> Option<Account> {
        self.account_repo.get_account()
    }

    fn update_2fa(&mut self, state: bool) -> AccountResponse {
        match self.account_repo.get_account() {
            None => trap("No such Account"),
            Some(mut acc) => {
                if !acc
                    .access_points
                    .clone()
                    .into_iter()
                    .any(|l| l.device_type.eq(&DeviceType::Passkey))
                {
                    trap("Forbidden")
                }
                acc.is2fa_enabled = state;
                self.account_repo.store_account(acc.clone());
                account_to_account_response(acc)
            }
        }
    }

    async fn create_account(
        &mut self,
        account_request: AccountRequest,
    ) -> HttpResponse<AccountResponse> {
        let princ = ic_service::get_caller().to_text();
        if ic_service::is_anonymous(princ.clone()) {
            return to_error_response("User is anonymous");
        }
        let mut devices: Vec<DeviceData> = Vec::default();
        let mut acc = account_request_to_account(account_request.clone());
        if account_request.email.is_some() {
            if !email_validation_service::contains(
                account_request.email.clone().expect("Failed to retrieve the email from the account request.").to_string(),
                princ,
            ) {
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
                None => trap("Device Data required"),
                Some(dd) => {
                    if !acc.principal_id.eq(&dd.pub_key) {
                        trap("Incorrect Device Data")
                    }
                    if !&dd.device_type.eq(&DeviceType::Email) {
                        trap("Only email device can be registered as a root")
                    }
                    acc.access_points
                        .insert(access_point_request_to_access_point(dd));
                }
            }
        } else {
            devices = ic_service::trap_if_not_authenticated(acc.anchor.clone(), get_caller()).await;
        }
        match { self.account_repo.create_account(acc.clone()) } {
            None => to_error_response("Impossible to link this II anchor, please try another one."),
            Some(_) => {
                let recovery_device = devices
                    .into_iter()
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

    fn remove_account(&mut self) -> HttpResponse<bool> {
        let result = self.account_repo.remove_account();
        if result.is_none() {
            return to_error_response("Unable to remove Account");
        }

        to_success_response(true)
    }

    fn get_account_by_anchor(
        &mut self,
        anchor: u64,
        wallet: WalletVariant,
    ) -> HttpResponse<AccountResponse> {
        match { self.account_repo.get_account_by_anchor(anchor, wallet) } {
            None => to_error_response("Anchor not registered."),
            Some(acc) => to_success_response(account_to_account_response(acc)),
        }
    }

    fn get_account_by_principal(&mut self, princ: String) -> HttpResponse<AccountResponse> {
        match { self.account_repo.get_account_by_principal(princ) } {
            None => to_error_response("Principal not registered."),
            Some(acc) => to_success_response(account_to_account_response(acc)),
        }
    }

    fn get_root_id_by_principal(&mut self, princ: String) -> Option<String> {
        match { self.account_repo.get_account_by_principal(princ) } {
            None => None,
            Some(acc) => Some(acc.principal_id),
        }
    }

    fn get_anchor_by_principal(&mut self, princ: String) -> Option<u64> {
        match { self.account_repo.get_account_by_principal(princ) } {
            None => None,
            Some(acc) => Some(acc.anchor),
        }
    }

    fn get_all_accounts(&mut self) -> Vec<Account> {
        self.account_repo.get_all_accounts()
    }

    async fn sync_recovery_phrase_from_internet_identity(
        &self,
        anchor: u64,
    ) -> HttpResponse<AccountResponse> {
        let devices = ic_service::trap_if_not_authenticated(anchor, ic_service::get_caller()).await;

        let account = match self
            .account_repo
            .get_account_by_anchor(anchor, WalletVariant::InternetIdentity)
        {
            None => {
                return to_error_response(
                    "There is no Internet Identity account by the anchor in Identity Manager.",
                )
            }
            Some(account) => account,
        };

        let account_response = devices
            .iter()
            .find(|device_data| device_data.key_type.eq(&ic_service::KeyType::SeedPhrase))
            .map(|device_data| {
                self.access_point_service
                    .migrate_recovery_device(device_data.clone(), &account)
            })
            .map(|account| to_success_response(account_to_account_response(account)))
            .unwrap_or_else(|| {
                to_error_response("The user has no recovery phrase in Internet Identity.")
            });

        account_response
    }
}
