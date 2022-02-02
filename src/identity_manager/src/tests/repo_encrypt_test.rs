use ic_cdk::export::Principal;
use crate::{AccessPoint, Configuration, ConfigurationRepo};
use crate::repository::encrypt::account_encrypt::{decrypt_access_point, decrypt_account, decrypt_persona, encrypt_access_point, encrypt_persona};
use crate::repo::{Account, AccountRepo, is_anchor_exists, Persona};
use crate::repository::encrypt::account_encrypt::encrypt_account;
use crate::repository::encrypted_repo::{EncryptedRepo};

#[test]
fn anchor_ex_test() {
    let a = Configuration {
        lambda: Principal::anonymous(),
        token_ttl: 0,
        key: [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
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
