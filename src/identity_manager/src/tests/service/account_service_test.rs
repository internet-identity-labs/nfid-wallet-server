#![feature(use_extern_macros)]

use std::sync::Arc;
use std::time::Duration;

use inject::{container, get};
use mockers::Scenario;

use crate::{AccountRepo, AccountService, AccountServiceTrait, AdminRepo, Configuration, PhoneNumberService, Principal};
use crate::repository::account_repo::AccountRepoTrait;
use crate::repository::encrypt::encrypted_repo::EncryptedRepo;
use crate::repository::phone_number_repo::PhoneNumberRepo;
use crate::repository::repo::Account;
use crate::service::ic_service;
use crate::tests::test_util;
use crate::tests::test_util::init_config;

use super::*;

#[test]
fn test_get_account_expect_acc_frm_trait() {
    let scenario = Scenario::new();
    let (mut cond, cond_handle) = scenario.create_mock_for::<dyn AccountRepoTrait>();
    let v = Account {
        anchor: 5,
        principal_id: "".to_string(),
        name: "".to_string(),
        phone_number: "".to_string(),
        access_points: vec![],
        personas: vec![],
    };
    scenario.expect(cond_handle.get_account().and_return(Some(v)));
    let mut acc_serv = AccountService {
        account_repo: cond,
        phone_number_service: PhoneNumberService { phone_number_repo: PhoneNumberRepo {} },
    };
    assert_eq!(5, acc_serv.get_account().data.unwrap().anchor)
}

#[test]
fn test_get_account_e2e() {
    init_config();
    let v = Account {
        anchor: 5,
        principal_id: ic_service::get_caller().to_text(),
        name: "".to_string(),
        phone_number: "".to_string(),
        access_points: vec![],
        personas: vec![],
    };
    EncryptedRepo::create_account(v);
    let ar = AccountRepo {};
    let mut acc_serv = AccountService {
        account_repo: ar,
        phone_number_service:
        PhoneNumberService { phone_number_repo: PhoneNumberRepo {} },
    };
    assert_eq!(5, acc_serv.get_account().data.unwrap().anchor);
    assert_eq!(ic_service::get_caller().to_text(), acc_serv.get_account().data.unwrap().principal_id)
}

