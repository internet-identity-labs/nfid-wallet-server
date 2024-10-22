use crate::repository::access_point_repo::AccessPointRepo;
use crate::service::access_point_service::AccessPointService;
use crate::service::account_service::AccountService;
use crate::service::persona_service::PersonaService;
use crate::{AccountRepo, ApplicationRepo, ApplicationService, PersonaRepo};
use inject::{container, get};

pub fn get_account_service() -> AccountService<AccountRepo, AccessPointService<AccessPointRepo>> {
    get!(&container![], AccountService<AccountRepo, AccessPointService<AccessPointRepo>>)
        .expect("No account service found in the container.")
}

pub fn get_persona_service(
) -> PersonaService<PersonaRepo, ApplicationService<ApplicationRepo, AccountRepo>> {
    get!(&container![], PersonaService<PersonaRepo, ApplicationService<ApplicationRepo, AccountRepo>>)
        .expect("No persona service found in the container.")
}

pub fn get_application_service() -> ApplicationService<ApplicationRepo, AccountRepo> {
    get!(&container![], ApplicationService<ApplicationRepo, AccountRepo>)
        .expect("No application service found in the container.")
}

pub fn get_access_point_service() -> AccessPointService<AccessPointRepo> {
    get!(&container![], AccessPointService<AccessPointRepo>)
        .expect("No access point service found in the container.")
}

pub fn get_account_repo() -> AccountRepo {
    get!(&container![], AccountRepo)
        .expect("No account repository found in the container.")
}
