

use std::collections::{HashSet};
use ic_cdk::{call, storage};
use ic_cdk::export::Principal;
use crate::{ConfigurationRepo, HttpResponse};
use crate::container_wrapper::get_account_repo;
use crate::repository::account_repo::AccountRepoTrait;
use crate::response_mapper::{to_error_response};

pub type AccountsToReplicate = HashSet<String>;
pub type HearthCount = u32;

#[deprecated()]
pub async fn flush() -> HttpResponse<bool> {
    let raw_keys = storage::get_mut::<AccountsToReplicate>();
    let keys = raw_keys
        .to_owned()
        .into_iter()
        .collect::<Vec<_>>();
    let account_repo = get_account_repo();
    let accounts = account_repo.get_accounts(keys);
    let canister_id = ConfigurationRepo::get().backup_canister_id.as_ref().unwrap();
    let resp: HttpResponse<bool> = match call(Principal::from_text(canister_id).unwrap(), "store_accounts", (accounts.clone(), 0)).await
    {
        Ok((res, )) => res,
        Err((_, err)) => to_error_response(&err.clone()),
    };
    raw_keys.clear();
    resp
}

#[deprecated()]
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



