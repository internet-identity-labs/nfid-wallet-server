use std::collections::HashSet;
use crate::AccountRepo;
use crate::repository::account_repo::{Account, AccountRepoTrait};
use crate::repository::repo::is_anchor_exists;
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
        base_fields: Default::default()
    };
    let ar = AccountRepo {};
    ar.store_account(acc);
    let a = is_anchor_exists(123);
    assert!(a)
}
