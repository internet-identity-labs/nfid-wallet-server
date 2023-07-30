use crate::{AccountRepo, AccountRequest, AccountService, AccountServiceTrait, AccountUpdateRequest, ApplicationRepo, ApplicationService, ic_service, PersonaRepo, PersonaRequest, PersonaService, PersonaServiceTrait};
use crate::http::requests::{WalletVariant};
use crate::repository::access_point_repo::AccessPointRepo;
use crate::repository::account_repo::{Account, AccountRepoTrait};
use crate::repository::persona_repo::Persona;
use crate::repository::phone_number_repo::PhoneNumberRepo;
use crate::repository::repo::BasicEntity;
use crate::service::access_point_service::AccessPointService;
use crate::tests::test_util::init_config;

#[async_std::test]
async fn test_get_account_e2e() {
    init_config();
    let persona = Persona {
        domain: "test".to_string(),
        persona_id: "test".to_string(),
        persona_name: None,
        base_fields: Default::default(),
        domain_certified: None,
    };
    let mut personas = vec![];
    personas.push(persona);
    let v = Account {
        anchor: 5,
        principal_id: ic_service::get_caller().to_text(),
        name: None,
        phone_number: None,
        phone_number_sha2: None,
        personas: personas,
        access_points: Default::default(),
        base_fields: BasicEntity::new(),
        wallet: WalletVariant::NFID,
        is2fa_enabled: false,
        email: None,
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
    let persona_request = PersonaRequest {
        domain: "test2".to_string(),
        persona_name: "test".to_string(),
        persona_id: "test2".to_string(),
    };
    let acc = AccountRepo {};
    let app = ApplicationRepo {};
    let persona_service = PersonaService {
        persona_repo: PersonaRepo { account_repo: acc },
        application_service: ApplicationService { account_repo: acc, application_repo: app },
    };
    let acc_upd = persona_service.update_persona(persona_request);

    let acc =  acc_serv.get_account().unwrap();
    let json_string = serde_json::to_string(&acc);
    let str = json_string.unwrap();
    let rest_acc: Account = serde_json::from_str(&str).unwrap();
    assert_eq!(true, acc_upd.error.is_some());
    assert_eq!(acc.base_fields, rest_acc.base_fields);
    assert_eq!(acc.anchor, rest_acc.anchor);
    assert_eq!(acc.principal_id, rest_acc.principal_id);

}

#[async_std::test]
async fn test_base_entity_on_account_create() {
    init_config();
    let v = AccountRequest {
        anchor: 10,
        wallet: None,
        access_point: None,
        email: None,
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
        anchor: 11,
        wallet: None,
        access_point: None,
        email: None,
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
        name: Option::from("321".to_string()),
        email: None,
    };
    acc_serv.update_account(vv);
    assert_eq!(123456789, acc_repo.get_account().unwrap().base_fields.get_modified_date());
    assert_eq!(123456789, acc_repo.get_account().unwrap().base_fields.get_created_date());
}
