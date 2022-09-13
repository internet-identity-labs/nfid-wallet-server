use std::collections::HashMap;
use ic_cdk::storage;
#[cfg(test)]
use mockers_derive::mocked;

#[cfg_attr(test, mocked)]
pub trait PhoneNumberRepoTrait {
    fn is_exist(&self, phone_number_encrypted: &String) -> bool;
    fn add(&self, phone_number_encrypted: String, principal: String) -> ();
    fn add_all(&self, phone_numbers_encrypted: HashMap<String, String>) -> ();
    fn remove(&self, phone_number_encrypted: &String) -> bool;
    fn get(&self, phone_number_encrypted: &String) -> Option<&String>;
}

pub type PhoneNumbers = HashMap<String, String>;

#[derive(Default)]
pub struct PhoneNumberRepo {}

impl PhoneNumberRepoTrait for PhoneNumberRepo {
    fn is_exist(&self, phone_number_encrypted: &String) -> bool {
        storage::get::<PhoneNumbers>().contains_key(phone_number_encrypted)
    }

    fn add(&self, phone_number_encrypted: String, principal: String) -> () {
        storage::get_mut::<PhoneNumbers>().insert(phone_number_encrypted, principal);
    }

    fn add_all(&self, phone_numbers_encrypted: HashMap<String, String>) -> () {
        storage::get_mut::<PhoneNumbers>().extend(phone_numbers_encrypted);
    }

    fn remove(&self, phone_number_encrypted: &String) -> bool {
        storage::get_mut::<PhoneNumbers>().remove(phone_number_encrypted).is_some()
    }

    fn get(&self, phone_number_encrypted: &String) -> Option<&String> {
        storage::get_mut::<PhoneNumbers>().get(phone_number_encrypted)
    }
}
