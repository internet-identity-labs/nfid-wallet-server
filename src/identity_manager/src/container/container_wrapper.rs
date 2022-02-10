use inject::{get, container};
use crate::{AccessPointRepo, PersonaRepo, AccessPointService, AccountRepo, AccountService, ApplicationRepo, ApplicationService, PhoneNumberService, PersonaService};
use crate::repository::phone_number_repo::PhoneNumberRepo;

pub fn get_account_service() -> AccountService<AccountRepo, PhoneNumberService<PhoneNumberRepo>> {
    get!(&container![], AccountService<AccountRepo, PhoneNumberService<PhoneNumberRepo>>).unwrap()
}

pub fn get_phone_number_service() -> PhoneNumberService<PhoneNumberRepo> {
    get!(&container![], PhoneNumberService<PhoneNumberRepo>).unwrap()
}

pub fn get_access_point_service() -> AccessPointService<AccessPointRepo> {
    get!(&container![], AccessPointService<AccessPointRepo>).unwrap()
}

pub fn get_persona_service() -> PersonaService<PersonaRepo, ApplicationService<ApplicationRepo, AccountRepo>> {
    get!(&container![], PersonaService<PersonaRepo, ApplicationService<ApplicationRepo, AccountRepo>>).unwrap()
}

pub fn get_application_service() -> ApplicationService<ApplicationRepo, AccountRepo> {
    get!(&container![], ApplicationService<ApplicationRepo, AccountRepo>).unwrap()
}