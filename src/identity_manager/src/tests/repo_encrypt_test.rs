use ic_cdk::export::Principal;
use crate::{Configuration, ConfigurationRepo};
use crate::repo::{Account, AccountRepo, is_anchor_exists};

#[test]
fn anchor_ex_test() {
    let a = Configuration {
        lambda: Principal::anonymous(),
        token_ttl: 0,
        token_refresh_ttl: 0,
        key: [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        whitelisted_phone_numbers: Option::None
    };
    ConfigurationRepo::save(a);
    let acc = Account {
        anchor: 123,
        principal_id: "".to_string(),
        name: "".to_string(),
        phone_number: "".to_string(),
        personas: vec![],
        access_points: vec![],
    };
    AccountRepo::store_account(acc);
    let a = is_anchor_exists(123);
    assert!(a)
}
