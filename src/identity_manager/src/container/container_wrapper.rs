use inject::{get, container};
use crate::{PersonaRepo, AccountRepo, AccountService, ApplicationRepo, ApplicationService, PhoneNumberService, PersonaService};
use crate::repository::access_point_repo::AccessPointRepo;
use crate::repository::phone_number_repo::PhoneNumberRepo;
use crate::repository::token_repo::TokenRepo;
use crate::service::access_point_service::AccessPointService;
use crate::service::credential_service::CredentialService;

pub fn get_account_service() -> AccountService<AccountRepo, PhoneNumberRepo, AccessPointService<AccessPointRepo>> {
    get!(&container![], AccountService<AccountRepo, PhoneNumberRepo, AccessPointService<AccessPointRepo>>).unwrap()
}

pub fn get_phone_number_service() -> PhoneNumberService<PhoneNumberRepo, TokenRepo, AccountRepo> {
    get!(&container![], PhoneNumberService<PhoneNumberRepo, TokenRepo, AccountRepo>).unwrap()
}

pub fn get_persona_service() -> PersonaService<PersonaRepo, ApplicationService<ApplicationRepo, AccountRepo>> {
    get!(&container![], PersonaService<PersonaRepo, ApplicationService<ApplicationRepo, AccountRepo>>).unwrap()
}

pub fn get_application_service() -> ApplicationService<ApplicationRepo, AccountRepo> {
    get!(&container![], ApplicationService<ApplicationRepo, AccountRepo>).unwrap()
}

pub fn get_credential_service() -> CredentialService<AccountRepo> {
    get!(&container![], CredentialService<AccountRepo>).unwrap()
}

pub fn get_access_point_service() -> AccessPointService<AccessPointRepo> {
    get!(&container![], AccessPointService<AccessPointRepo>).unwrap()
}

pub fn get_account_repo() -> AccountRepo<> {
    get!(&container![], AccountRepo<>).unwrap()
}
