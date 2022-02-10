use std::time::Duration;
use ic_cdk::export::Principal;
use crate::{Configuration, ConfigurationRepo};

pub(crate) fn init_config(){
    let a = Configuration {
        lambda: Principal::anonymous(),
        token_ttl: Duration::from_secs(0),
        token_refresh_ttl: Duration::from_secs(0),
        key: [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        whitelisted: Vec::default()
    };
    ConfigurationRepo::save(a);
}