use std::time::Duration;
use ic_cdk::export::Principal;
use crate::{AccountRequest, AccountServiceTrait, Configuration, ConfigurationRepo, get_account_service};

pub fn init_config(){
    // let a = Configuration {
    //     lambda: Principal::anonymous(),
    //     token_ttl: Duration::from_secs(0),
    //     token_refresh_ttl: Duration::from_secs(0),
    //     whitelisted_phone_numbers: Vec::default(),
    //     heartbeat: Option::None,
    //     backup_canister_id: Option::Some("rrkah-fqaaa-aaaaa-aaaaq-cai".to_string()),
    //     whitelisted_canisters: vec![],
    //     env: None
    // };
    // ConfigurationRepo::save(a);
}

pub fn create_default_account(){
    let acc = AccountRequest {
        anchor: 5,
    };
    let mut account_service = get_account_service();
    account_service.create_account(acc);
}