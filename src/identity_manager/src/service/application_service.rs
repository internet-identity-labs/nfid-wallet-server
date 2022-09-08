use std::collections::HashSet;
use crate::{HttpResponse};
use crate::repository::account_repo::{Account, AccountRepoTrait};
use crate::repository::application_repo::{Application, ApplicationRepoTrait};
use crate::response_mapper::{to_error_response, to_success_response};

pub trait ApplicationServiceTrait {
    fn read_applications(&self) -> HttpResponse<Vec<Application>>;
    fn get_application_by_domain(&self, domain: String) -> HttpResponse<Application>;
    fn delete_application(&self, name: String) -> HttpResponse<bool>;
    fn update_application(&self, app: Application) -> HttpResponse<Vec<Application>>;
    fn update_application_alias(&self, alias: String, domain: String) -> HttpResponse<bool>;
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

    fn get_application_by_domain(&self, domain: String) -> HttpResponse<Application> {
        return match self.application_repo.get_application(&domain) {
            None => {
                to_error_response("Unable to fina application.")
            }
            Some(app) => {
                to_success_response(app.to_owned())
            }
        };
    }

    fn delete_application(&self, domain: String) -> HttpResponse<bool> {
        let apps = self.application_repo.delete_application(domain);
        if apps {
            return to_success_response(apps);
        }
        to_error_response("Unable to remove app with such name.")
    }

    fn update_application_alias(&self, domain: String, alias: String) -> HttpResponse<bool> {
        let apps = self.application_repo.get_application(&domain);
        return match apps {
            None => {
                let mut aliases = HashSet::new();
                aliases.insert(alias.clone());
                let app = Application {
                    domain,
                    user_limit: 5,
                    alias: Some(aliases),
                    img: None,
                    name: alias,
                };
                self.application_repo.create_application(app);
                to_success_response(true)
            }
            Some(app) => {
                return match app.alias {
                    None => {
                        let mut aliases = HashSet::new();
                        aliases.insert(alias.clone());
                        let mut updated_app = app.clone();
                        updated_app.alias = Some(aliases);
                        let resp = self.application_repo.update_application(updated_app);
                        to_success_response(resp)
                    }
                    Some(_) => {
                        let mut updated_app = app.clone();
                        let mut aliases = app.alias.clone().unwrap();
                        aliases.insert(alias);
                        updated_app.alias = Some(aliases);
                        let resp = self.application_repo.update_application(updated_app);
                        to_success_response(resp)
                    }
                };
            }
        };
    }

    fn update_application(&self, new_app: Application) -> HttpResponse<Vec<Application>> {
        let apps = self.application_repo.get_application(&new_app.domain);
        return match apps {
            None => {
                to_error_response("Unable to update Application. Application not exists")
            }
            Some(_) => {
                self.application_repo.update_application(new_app);
                to_success_response(self.application_repo.read_applications())
            }
        };
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
        let default_limit = 5;
        match self.account_repo.get_account() {
            None => { false }
            Some(acc) => {
                self.application_repo.read_applications()
                    .into_iter()
                    .find(|l| l.domain.eq(domain))
                    .map(|x| x.user_limit)
                    .map(|x| compare_limits(&acc, &domain, x))
                    .unwrap_or(compare_limits(&acc, &domain, default_limit))
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