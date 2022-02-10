use ic_cdk::storage;

use crate::Application;
use crate::repository::repo::Applications;

pub trait ApplicationRepoTrait {
    fn create_application(&self, application: Application) -> Vec<Application>;
    fn read_applications(&self) -> Vec<Application>;
    fn delete_application(&self, name: String) -> bool;
    fn is_application_exists(&self, application: &Application) -> bool;
}

#[derive(Default)]
pub struct ApplicationRepo {}

impl ApplicationRepoTrait for ApplicationRepo {
    fn create_application(&self, application: Application) -> Vec<Application> {
        let applications = storage::get_mut::<Applications>();
        applications.insert(application.clone());
        applications.iter()
            .map(|p| p.clone())
            .collect()
    }

    fn read_applications(&self) -> Vec<Application> {
        storage::get_mut::<Applications>().iter()
            .map(|p| p.clone())
            .collect()
    }

    fn delete_application(&self, name: String) -> bool {
        let app_to_remove = storage::get_mut::<Applications>().iter()
            .find(|a| a.name.eq(&name));
        match app_to_remove {
            None => { false }
            Some(app) => {
                storage::get_mut::<Applications>()
                    .remove(app)
            }
        }
    }

    fn is_application_exists(&self, application: &Application) -> bool {
        let applications = storage::get_mut::<Applications>();
        applications.iter()
            .any(|a| a.name.eq(&application.name) || a.domain.eq(&application.domain))
    }
}