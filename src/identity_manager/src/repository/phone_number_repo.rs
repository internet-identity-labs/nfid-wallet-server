use std::collections::HashSet;
use ic_cdk::storage;
#[cfg(test)]
use mockers_derive::mocked;

#[cfg_attr(test, mocked)]
pub trait PhoneNumberRepoTrait {
    fn is_exist(&self, phone_number_encrypted: &String) -> bool;
    fn add(&self, phone_number_encrypted: String) -> ();
    fn add_all(&self, phone_numbers_encrypted: HashSet<String>) -> ();
    fn remove(&self, phone_number_encrypted: &String) -> bool;
}

pub type PhoneNumbers = HashSet<String>;

#[derive(Default)]
pub struct PhoneNumberRepo {}

impl PhoneNumberRepoTrait for PhoneNumberRepo {
    fn is_exist(&self, phone_number_encrypted: &String) -> bool {
        storage::get::<PhoneNumbers>().contains(phone_number_encrypted)
    }

    fn add(&self, phone_number_encrypted: String) -> () {
        storage::get_mut::<PhoneNumbers>().insert(phone_number_encrypted);
    }

    fn add_all(&self, phone_numbers_encrypted: HashSet<String>) -> () {
        storage::get_mut::<PhoneNumbers>().extend(phone_numbers_encrypted);
    }

    fn remove(&self, phone_number_encrypted: &String) -> bool {
        storage::get_mut::<PhoneNumbers>().remove(phone_number_encrypted)
    }
}
