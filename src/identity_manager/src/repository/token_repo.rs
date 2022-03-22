use std::collections::HashMap;
use std::time::Duration;
use ic_cdk::storage;
#[cfg(test)]
use mockers_derive::mocked;

#[cfg_attr(test, mocked)]
pub trait TokenRepoTrait {
    fn add(&self, principal_id_encrypted: String, token_encrypted: String, phone_number_encrypted: String) -> ();
    fn get(&self, principal_id_encrypted: &String, duration: Duration) -> Option<(&String, &String)>;
}

#[derive(Default)]
pub struct TokenRepo {}

pub type Tokens = HashMap<String, (String, String, u64)>;

impl TokenRepoTrait for TokenRepo {
    #[cfg(not(test))]  //todo TEMP move to ic_service
    fn add(&self, principal_id_encrypted: String, token_encrypted: String, phone_number_encrypted: String) -> () {
        let time_window: u64 = ic_cdk::api::time() - crate::ConfigurationRepo::get().token_ttl.as_nanos() as u64;
        storage::get_mut::<Tokens>().retain(|_, (_, _, t)| *t > time_window);

        let value = (token_encrypted, phone_number_encrypted, ic_cdk::api::time());
        storage::get_mut::<Tokens>().insert(principal_id_encrypted, value);
    }
    #[cfg(test)]
    fn add(&self, principal_id_encrypted: String, token_encrypted: String, phone_number_encrypted: String) -> () {
        let value = (token_encrypted, phone_number_encrypted, 123);
        storage::get_mut::<Tokens>().insert(principal_id_encrypted, value);
    }
    #[cfg(not(test))]
    fn get(&self, principal_id_encrypted: &String, duration: Duration) -> Option<(&String, &String)> {
        let time_window: u64 = ic_cdk::api::time() - duration.as_nanos() as u64;
        storage::get::<Tokens>()
            .get(principal_id_encrypted)
            .filter(|(_, _, t)| *t > time_window)
            .map(|(token, phone_number, _)| (token, phone_number))
    }
    #[cfg(test)]
    fn get(&self, principal_id_encrypted: &String, _duration: Duration) -> Option<(&String, &String)> {
        storage::get::<Tokens>()
            .get(principal_id_encrypted)
            .map(|(token, phone_number, _)| (token, phone_number))
    }
}
