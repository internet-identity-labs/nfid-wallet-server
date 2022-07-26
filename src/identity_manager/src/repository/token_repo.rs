use std::collections::HashMap;
use std::time::Duration;
use ic_cdk::storage;
#[cfg(test)]
use mockers_derive::mocked;

#[cfg_attr(test, mocked)]
pub trait TokenRepoTrait {
    fn add(&self, principal_id: String, token_encrypted: String, phone_number_encrypted: String, phone_number_hash: String) -> ();
    fn get(&self, principal_id: &String, duration: Duration) -> Option<(&String, &String, &String)>;
}

#[derive(Default)]
pub struct TokenRepo {}

pub type Tokens = HashMap<String, (String, String, String, u64)>;  //todo wrap with object

impl TokenRepoTrait for TokenRepo {
    #[cfg(not(test))]  //todo TEMP move to ic_service
    fn add(&self, principal_id_encrypted: String, token_encrypted: String, phone_number_encrypted: String, phone_number_hash: String) -> () {
        let time_window: u64 = ic_cdk::api::time() - crate::ConfigurationRepo::get().token_ttl.as_nanos() as u64;
        storage::get_mut::<Tokens>().retain(|_, (_, _, _, t)| *t > time_window);

        let value = (token_encrypted, phone_number_encrypted, phone_number_hash, ic_cdk::api::time());
        storage::get_mut::<Tokens>().insert(principal_id_encrypted, value);
    }
    #[cfg(test)]
    fn add(&self, principal_id_encrypted: String, token_encrypted: String, phone_number_encrypted: String, phone_number_hash: String) -> () {
        let value = (token_encrypted, phone_number_encrypted, phone_number_hash, 123);
        storage::get_mut::<Tokens>().insert(principal_id_encrypted, value);
    }
    #[cfg(not(test))]
    fn get(&self, principal_id_encrypted: &String, duration: Duration) -> Option<(&String, &String, &String)> {
        let time_window: u64 = ic_cdk::api::time() - duration.as_nanos() as u64;
        storage::get::<Tokens>()
            .get(principal_id_encrypted)
            .filter(|(_, _, _, t)| *t > time_window)
            .map(|(token, phone_number_encrypted, phone_number_hash, _)| (token, phone_number_encrypted, phone_number_hash))
    }
}
