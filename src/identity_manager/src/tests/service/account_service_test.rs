use crate::{AccountRepo, AccountRequest, AccountService, AccountServiceTrait, AccountUpdateRequest, ic_service};
use crate::repository::account_repo::{Account, AccountRepoTrait};
use crate::repository::phone_number_repo::{PhoneNumberRepo};
use crate::repository::repo::BasicEntity;

use crate::tests::test_util::init_config;
use async_std::test;
use crate::repository::access_point_repo::AccessPointRepo;
use crate::service::access_point_service::AccessPointService;

// #[test]
// fn test_get_account_expect_acc_frm_trait() {
//     let scenario = Scenario::new();
//     let (cond, cond_handle) = scenario.create_mock_for::<dyn AccountRepoTrait>();
//     let v = Account {
//         anchor: 5,
//         principal_id: "".to_string(),
//         name: None,
//         phone_number: None,
//         personas: vec![],
//         access_points: Default::default(),
//         base_fields: BasicEntity::new(),
//     };
//     scenario.expect(cond_handle.get_account().and_return(Some(v)));
//     let mut acc_serv = AccountService {
//         account_repo: cond,
//         phone_number_repo: PhoneNumberRepo {},
//     };
//     assert_eq!(5, acc_serv.get_account().data.unwrap().anchor);
// }

#[async_std::test]
async fn test_get_account_e2e() {
    init_config();
    let v = Account {
        anchor: 5,
        principal_id: ic_service::get_caller().to_text(),
        name: None,
        phone_number: None,
        phone_number_sha2: None,
        personas: vec![],
        access_points: Default::default(),
        base_fields: BasicEntity::new(),
    };
    let ar = AccountRepo {};
    ar.create_account(v);
    let mut acc_serv = AccountService {
        account_repo: ar,
        phone_number_repo: PhoneNumberRepo {},
        access_point_service: AccessPointService { access_point_repo: AccessPointRepo { account_repo: ar } },
    };
    assert_eq!(5, acc_serv.get_account().unwrap().anchor);
    assert_eq!(ic_service::get_caller().to_text(), acc_serv.get_account().unwrap().principal_id);
}

#[async_std::test]
async fn test_base_entity_on_account_create() {
    init_config();
    let v = AccountRequest {
        anchor: 10
    };
    let ar = AccountRepo {};
    let mut acc_serv = AccountService {
        account_repo: ar,
        phone_number_repo: PhoneNumberRepo {},
        access_point_service: AccessPointService { access_point_repo: AccessPointRepo { account_repo: ar } },
    };
    let acc_repo = AccountRepo {};
    let anch = acc_serv.create_account(v).await.data.unwrap().anchor;
    assert_eq!(10, anch);
    assert_eq!(123456789, acc_repo.get_account().unwrap().base_fields.get_modified_date());
    assert_eq!(123456789, acc_repo.get_account().unwrap().base_fields.get_created_date());
}

#[async_std::test]
async fn test_base_entity_on_account_update() {
    init_config();
    let v = AccountRequest {
        anchor: 11
    };
    let ar = AccountRepo {};
    let mut acc_serv = AccountService {
        account_repo: ar,
        phone_number_repo: PhoneNumberRepo {},
        access_point_service: AccessPointService { access_point_repo: AccessPointRepo { account_repo: ar } },
    };
    let acc_repo = AccountRepo {};
    assert_eq!(11, acc_serv.create_account(v).await.data.unwrap().anchor);
    let vv = AccountUpdateRequest {
        name: Option::from("321".to_string())
    };
    acc_serv.update_account(vv);
    assert_eq!(123456789, acc_repo.get_account().unwrap().base_fields.get_modified_date());
    assert_eq!(123456789, acc_repo.get_account().unwrap().base_fields.get_created_date());
}
