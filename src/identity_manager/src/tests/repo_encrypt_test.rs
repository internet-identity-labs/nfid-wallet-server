use std::time::Duration;
use ic_cdk::export::Principal;
use inject::{get, container};
use crate::{AccountRepo, Configuration, ConfigurationRepo};
use crate::account_service::{AccountServiceTrait, AccountService};
use crate::repository::account_repo::AccountRepoTrait;
use crate::repository::repo::{Account, is_anchor_exists};
use crate::tests::test_util::init_config;

#[test]
fn anchor_ex_test() {
    init_config();
    let acc = Account {
        anchor: 123,
        principal_id: "".to_string(),
        name: "".to_string(),
        phone_number: "".to_string(),
        personas: vec![],
        access_points: vec![],
    };
    let ar = AccountRepo {};
    ar.store_account(acc);
    let a = is_anchor_exists(123);
    assert!(a)
}
