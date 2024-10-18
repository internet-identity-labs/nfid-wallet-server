use crate::repository::account_repo::AccountRepoTrait;
use crate::repository::application_repo::{Application, ApplicationRepoTrait};
use crate::response_mapper::to_success_response;
use crate::HttpResponse;

#[deprecated()]
pub trait ApplicationServiceTrait {
    fn read_applications(&self) -> HttpResponse<Vec<Application>>;
}

#[derive(Default)]
pub struct ApplicationService<T, N> {
    pub account_repo: N,
    pub application_repo: T,
}

impl<T: ApplicationRepoTrait, N: AccountRepoTrait> ApplicationServiceTrait
    for ApplicationService<T, N>
{
    fn read_applications(&self) -> HttpResponse<Vec<Application>> {
        let apps = self.application_repo.read_applications();
        to_success_response(apps)
    }
}
