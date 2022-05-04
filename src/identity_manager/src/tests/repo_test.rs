use crate::{AccountRepo, Principal};
use crate::repository::account_repo::{Account, AccountRepoTrait};
use crate::repository::repo::is_anchor_exists;
use crate::tests::test_util::init_config;

#[test]
fn anchor_ex_test() {
    init_config();
    let acc = Account {
        anchor: 123,
        principal_id: "".to_string(),
        name: None,
        phone_number: None,
        phone_number_sha2: None,
        personas: vec![],
        access_points: Default::default(),
        base_fields: Default::default()
    };
    let ar = AccountRepo {};
    ar.store_account(acc);
    let a = is_anchor_exists(123);
    let t = Principal::from_text("qoctq-giaaa-aaaaa-aaaea-cai").unwrap();
    assert!(a)
}
