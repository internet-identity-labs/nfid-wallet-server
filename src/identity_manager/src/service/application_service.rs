use crate::HttpResponse;
use crate::repo::{Account, AccountRepo, Application, ApplicationRepo};
use crate::response_mapper::{to_error_response, to_success_response};

pub fn read_applications() -> HttpResponse<Vec<Application>> {
    let apps = ApplicationRepo::read_applications();
    to_success_response(apps)
}

pub fn delete_application(name: String) -> HttpResponse<bool> {
    let apps = ApplicationRepo::delete_application(name);
    if apps {
        return to_success_response(apps);
    }
    to_error_response("Unable to remove app with such name.")
}

pub fn create_application(app: Application) -> HttpResponse<Vec<Application>> {
    if ApplicationRepo::is_application_exists(&app) {
        return to_error_response("Unable to create Application. Application exists");
    }
    let apps = ApplicationRepo::create_application(app);
    to_success_response(apps)
}

pub fn is_over_the_application_limit(domain: &String) -> HttpResponse<bool> {
    to_success_response(is_over_the_limit(domain))
}

pub fn is_over_the_limit(domain: &String) -> bool {
    match AccountRepo::get_account() {
        None => { false }
        Some(acc) => {
            ApplicationRepo::read_applications()
                .into_iter()
                .find(|l| l.domain.eq(domain))
                .map(|x| x.user_limit)
                .map(|x| compare_limits(&acc, &domain, x))
                .unwrap_or(false)
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
