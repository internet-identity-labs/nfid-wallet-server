use crate::{HttpResponse};
use crate::repository::account_repo::{Account, AccountRepoTrait};
use crate::repository::application_repo::{Application, ApplicationRepoTrait};
use crate::response_mapper::{to_error_response, to_success_response};

pub trait ApplicationServiceTrait {
    fn read_applications(&self) -> HttpResponse<Vec<Application>>;
    fn delete_application(&self, name: String) -> HttpResponse<bool>;
    fn create_application(&self, app: Application) -> HttpResponse<Vec<Application>>;
    fn is_over_the_application_limit(&self, domain: &String) -> HttpResponse<bool>;
    fn is_over_the_limit(&self, domain: &String) -> bool;
}

#[derive(Default)]
pub struct ApplicationService<T, N> {
    pub account_repo: N,
    pub application_repo: T,
}

impl<T: ApplicationRepoTrait, N: AccountRepoTrait> ApplicationServiceTrait for ApplicationService<T, N> {
    fn read_applications(&self) -> HttpResponse<Vec<Application>> {
        let apps = self.application_repo.read_applications();
        to_success_response(apps)
    }

    fn delete_application(&self, name: String) -> HttpResponse<bool> {
        let apps = self.application_repo.delete_application(name);
        if apps {
            return to_success_response(apps);
        }
        to_error_response("Unable to remove app with such name.")
    }

    fn create_application(&self, app: Application) -> HttpResponse<Vec<Application>> {
        if self.application_repo.is_application_exists(&app) {
            return to_error_response("Unable to create Application. Application exists");
        }
        let apps = self.application_repo.create_application(app);
        to_success_response(apps)
    }

    fn is_over_the_application_limit(&self, domain: &String) -> HttpResponse<bool> {
        to_success_response(self.is_over_the_limit(domain))
    }

    fn is_over_the_limit(&self, domain: &String) -> bool {
        match self.account_repo.get_account() {
            None => { false }
            Some(acc) => {
                self.application_repo.read_applications()
                    .into_iter()
                    .find(|l| l.domain.eq(domain))
                    .map(|x| x.user_limit)
                    .map(|x| compare_limits(&acc, &domain, x))
                    .unwrap_or(false)
            }
        }
    }
}

fn compare_limits(account: &Account, domain: &String, limit: u16) -> bool {
    let current_count = account.personas
        .iter()
        .map(|p| p.domain.clone())
        .filter(|d| d.to_lowercase().eq(&domain.to_lowercase()))
        .count();
    (current_count as u16) >= limit
}