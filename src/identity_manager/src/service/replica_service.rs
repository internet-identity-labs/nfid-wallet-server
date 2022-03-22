use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use ic_cdk::{call, print, storage, trap};
use ic_cdk::export::Principal;
use crate::{AccountService, AccountServiceTrait, ConfigurationRepo, HttpResponse, to_success_response};
use crate::container_wrapper::get_account_repo;
use crate::repository::account_repo::AccountRepoTrait;
use crate::response_mapper::{to_error_http_response, to_error_response};

pub type AccountsToReplicate = HashSet<String>;
pub type HearthCount = u32;


pub async fn flush() -> HttpResponse<bool> {
    let mut raw_keys = storage::get_mut::<AccountsToReplicate>();
    let keys = raw_keys
        .to_owned()
        .into_iter()
        .collect::<Vec<_>>();
    let account_repo = get_account_repo();
    let accounts = account_repo.get_accounts(keys);
    let mut canister_id = &ConfigurationRepo::get().backup_canister_id;
    let resp: HttpResponse<bool> = match call(Principal::from_text(canister_id).unwrap(), "store_accounts", (accounts.clone(), 0)).await
    {
        Ok((res, )) => res,
        Err((_, err)) => to_error_response(&err.clone()),
    };
    raw_keys.clear();
    resp
}

pub async fn restore_and_flush(canister_id: String) -> HttpResponse<bool> {
    let account_repo = get_account_repo();
    let accounts = account_repo.get_all_accounts();
    let resp: HttpResponse<bool> = match call(Principal::from_text(canister_id).unwrap(), "store_accounts", (accounts.clone(), 0)).await
    {
        Ok((res, )) => res,
        Err((_, err)) => to_error_response(&err.clone()),
    };
    resp
}



