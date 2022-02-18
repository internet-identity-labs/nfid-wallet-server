use std::time::Duration;
use ic_cdk::api::time;
use ic_cdk::storage;
use crate::{ConfigurationRepo};
use crate::repository::repo::Tokens;
#[cfg(test)]
use mockers_derive::mocked;

#[cfg_attr(test, mocked)]
pub trait TokenRepoTrait {
    fn add(&self, phone_number: blake3::Hash, token: blake3::Hash) -> ();
    fn get(&self, phone_number: &blake3::Hash, duration: Duration) -> Option<&blake3::Hash>;
}

#[derive(Default)]
pub struct TokenRepo {}

impl TokenRepoTrait for TokenRepo {
    #[cfg(not(test))]  //todo TEMP move to ic_service
    fn add(&self, phone_number: blake3::Hash, token: blake3::Hash) -> () {
        let time_window: u64 = time() - ConfigurationRepo::get().token_ttl.as_nanos() as u64;
        storage::get_mut::<Tokens>().retain(|_, (_, t)| *t > time_window);
        storage::get_mut::<Tokens>().insert(phone_number, (token, time()));
    }
    #[cfg(not(test))]
    fn get(&self, phone_number: &blake3::Hash, duration: Duration) -> Option<&blake3::Hash> {
        let time_window: u64 = time() - duration.as_nanos() as u64;
        storage::get::<Tokens>()
            .get(phone_number)
            .filter(|(_, t)| *t > time_window)
            .map(|(v, _)| v)
    }
    #[cfg(test)]
    fn add(&self, phone_number: blake3::Hash, token: blake3::Hash) -> () {
        storage::get_mut::<Tokens>().insert(phone_number, (token, 123));
    }
    #[cfg(test)]
    fn get(&self, phone_number: &blake3::Hash, duration: Duration) -> Option<&blake3::Hash> {
        storage::get::<Tokens>()
            .get(phone_number)
            .map(|(v, _)| v)
    }
}