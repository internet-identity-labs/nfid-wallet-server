use ic_cdk::export::Principal;
use ic_cdk::storage;
use serde_bytes::ByteBuf;
use crate::{AccessPointRequest, AccessPointServiceTrait, AccountService, AccountServiceTrait, create_access_point, get_access_point_service, get_account, get_account_service, ic_service};
use crate::repository::access_point_repo::AccessPoint;
use crate::repository::encrypt::account_encrypt::encrypt;
use crate::repository::encrypt::encrypted_repo::{EncryptedRepo, PrincipalIndex};
use crate::tests::test_util::{create_default_account, init_config};

#[test]
fn test_ap_e2e() {
    init_config();
    create_default_account();
    let pk = ByteBuf::from([4, 77, 158, 238, 70, 147, 57, 109, 238, 171, 6, 152, 247, 30, 197, 122, 30, 125, 165, 33, 73, 214, 134, 170, 45, 69, 147, 218, 142, 108, 224, 31, 71, 46, 190, 0, 97, 34, 118, 19, 80, 104, 19, 95, 184, 40, 48, 217, 33, 105, 10, 208, 8, 190, 188, 207, 5, 159, 66, 162, 180, 238, 142, 175, 205]);
    let ap = AccessPointRequest { pub_key: pk.clone() };
    let mut account_service = get_access_point_service();
    account_service.create_access_point(ap);
    let pr = Principal::self_authenticating(pk.clone()).to_text();
    let encrypted_princ = encrypt(pr.to_owned());
    let index = storage::get_mut::<PrincipalIndex>();
    let mut account_service = get_account_service();
    let acc = account_service.get_account();
    assert_eq!(acc.data.unwrap().access_points.len(), 1);
    assert_eq!(index.into_iter().len(), 2);
    match index.get_mut(&encrypted_princ) {
        None => {
            assert!(false)
        }
        Some(k) => {
            assert_eq!(encrypt(ic_service::get_caller().to_text()), k.to_owned())
        }
    }
}
