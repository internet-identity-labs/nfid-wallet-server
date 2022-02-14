use std::collections::HashSet;
use mockers::Scenario;
use crate::{AccountRepo, AccountRequest, AccountService, AccountServiceTrait, AccountUpdateRequest, HTTPVerifyPhoneNumberRequest, ic_service, PhoneNumberService, PhoneNumberServiceTrait};
use crate::repository::account_repo::{Account, AccountRepoTrait};
use crate::repository::encrypt::encrypted_repo::EncryptedRepo;
use crate::repository::phone_number_repo::{PhoneNumberRepo};
use crate::repository::repo::BasicEntity;
use crate::repository::token_repo::{TokenRepo};
use crate::tests::test_util::init_config;

#[test]
fn test_get_account_expect_acc_frm_trait() {
    let scenario = Scenario::new();
    let (cond, cond_handle) = scenario.create_mock_for::<dyn AccountRepoTrait>();
    let v = Account {
        anchor: 5,
        principal_id: "".to_string(),
        name: "".to_string(),
        phone_number: "".to_string(),
        personas: vec![],
        base_fields: BasicEntity::new(),
    };
    scenario.expect(cond_handle.get_account().and_return(Some(v)));
    let mut acc_serv = AccountService {
        account_repo: cond,
        phone_number_service: PhoneNumberService {
            phone_number_repo: PhoneNumberRepo {},
            token_repo: TokenRepo {},
        },
    };
    assert_eq!(5, acc_serv.get_account().data.unwrap().anchor);
}

#[test]
fn test_get_account_e2e() {
    init_config();
    let v = Account {
        anchor: 5,
        principal_id: ic_service::get_caller().to_text(),
        name: "".to_string(),
        phone_number: "".to_string(),
        personas: vec![],
        base_fields: BasicEntity::new(),
    };
    EncryptedRepo::create_account(v);
    let ar = AccountRepo {};
    let mut acc_serv = AccountService {
        account_repo: ar,
        phone_number_service:
        PhoneNumberService { phone_number_repo: PhoneNumberRepo {}, token_repo: TokenRepo {} },
    };
    assert_eq!(5, acc_serv.get_account().data.unwrap().anchor);
    assert_eq!(ic_service::get_caller().to_text(), acc_serv.get_account().data.unwrap().principal_id);
    assert_eq!(true, acc_serv.remove_account().data.unwrap());
    assert_eq!("Unable to remove Account", acc_serv.remove_account().error.unwrap());
}

#[test]
fn test_base_entity_on_account_create() {
    init_config();
    let v = AccountRequest {
        anchor: 5,
        name: "123".to_string(),
        phone_number: "321".to_string(),
        token: "123".to_string(),
    };
    let ar = AccountRepo {};
    let req = HTTPVerifyPhoneNumberRequest { phone_number: "321".to_string(), token: "123".to_string() };
    let ps = PhoneNumberService { phone_number_repo: PhoneNumberRepo {}, token_repo: TokenRepo {} };
    ps.post_token(req);
    let mut acc_serv = AccountService {
        account_repo: ar,
        phone_number_service: ps,
    };
    let acc_repo = AccountRepo {};
    assert_eq!(5, acc_serv.create_account(v).data.unwrap().anchor);
    assert_eq!(123456789, EncryptedRepo::get_account().unwrap().base_fields.get_created_date());
    assert_eq!(123456789, EncryptedRepo::get_account().unwrap().base_fields.get_modified_date());
    assert_eq!(123456789, acc_repo.get_account().unwrap().base_fields.get_modified_date());
    assert_eq!(123456789, acc_repo.get_account().unwrap().base_fields.get_created_date());
}

#[test]
fn test_base_entity_on_account_update() {
    init_config();
    let v = AccountRequest {
        anchor: 5,
        name: "123".to_string(),
        phone_number: "321".to_string(),
        token: "123".to_string(),
    };
    let ar = AccountRepo {};
    let req = HTTPVerifyPhoneNumberRequest { phone_number: "321".to_string(), token: "123".to_string() };
    let ps = PhoneNumberService { phone_number_repo: PhoneNumberRepo {}, token_repo: TokenRepo {} };
    ps.post_token(req);
    let mut acc_serv = AccountService {
        account_repo: ar,
        phone_number_service: ps,
    };
    let acc_repo = AccountRepo {};
    assert_eq!(5, acc_serv.create_account(v).data.unwrap().anchor);
    assert_eq!(123456789, EncryptedRepo::get_account().unwrap().base_fields.get_created_date());
    assert_eq!(123456789, EncryptedRepo::get_account().unwrap().base_fields.get_modified_date());
    let vv = AccountUpdateRequest {
        name: Option::from("321".to_string())
    };
    acc_serv.update_account(vv);
    assert_eq!(123456789, acc_repo.get_account().unwrap().base_fields.get_modified_date());
    assert_eq!(123456789, acc_repo.get_account().unwrap().base_fields.get_created_date());
}
