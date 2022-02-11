use std::collections::HashSet;
use ic_cdk::storage;
use crate::repository::repo::PhoneNumbers;
#[cfg(test)]
use mockers_derive::mocked;

#[cfg_attr(test, mocked)]
pub trait PhoneNumberRepoTrait {
    fn is_exist(&self, phone_number_hash: &blake3::Hash) -> bool;
    fn add(&self, phone_number_hash: blake3::Hash) -> ();
    fn add_all(&self, phone_number_hashes: HashSet<blake3::Hash>) -> ();
}

#[derive(Default)]
pub struct PhoneNumberRepo {}

impl PhoneNumberRepoTrait for PhoneNumberRepo {
    fn is_exist(&self, phone_number_hash: &blake3::Hash) -> bool {
        storage::get::<PhoneNumbers>().contains(phone_number_hash)
    }

    fn add(&self, phone_number_hash: blake3::Hash) -> () {
        storage::get_mut::<PhoneNumbers>().insert(phone_number_hash);
    }

    fn add_all(&self, phone_number_hashes: HashSet<blake3::Hash>) -> () {
        storage::get_mut::<PhoneNumbers>().extend(phone_number_hashes);
    }
}