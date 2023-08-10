use std::time::Duration;
use ic_cdk::export::Principal;
use crate::{AccountRequest, AccountServiceTrait, Configuration, ConfigurationRepo, get_account_service};

pub fn init_config(){
    let a = Configuration {
        lambda_url: "https://d8m9ttp390ku4.cloudfront.net/dev".to_string(),
        lambda: Principal::anonymous(),
        token_ttl: Duration::from_secs(0),
        token_refresh_ttl: Duration::from_secs(0),
        whitelisted_phone_numbers: Vec::default(),
        heartbeat: Option::None,
        backup_canister_id: Option::Some("rrkah-fqaaa-aaaaa-aaaaq-cai".to_string()),
        ii_canister_id: Principal::anonymous(),
        whitelisted_canisters: Option::None,
        env: Option::Some("test".to_string()),
        git_branch: None,
        commit_hash: None
    };
    ConfigurationRepo::save(a);
}

pub(crate) async fn create_default_account(){
    let acc = AccountRequest {
        anchor: 5,
        wallet: None,
        access_point: None,
        email: None,
    };
    let mut account_service = get_account_service();
    account_service.create_account(acc).await;
}