use std::collections::HashSet;
use std::iter::FromIterator;
use blake3::Hash;
use crate::PHONE_NUMBER_INDEX;
use crate::repository::repo::Account;

pub fn is_exist(phone_number_hash: &Hash) -> Result<(), &str> {
    PHONE_NUMBER_INDEX.with(|index| {
        return match index.borrow().get(&phone_number_hash) {
            Some(_) => Err("Phone number already exists"),
            None => Ok(())
        };
    })
}

pub fn add(phone_number_hash: Hash) -> () {
    PHONE_NUMBER_INDEX.with(|index| {
        index.borrow_mut().insert(phone_number_hash);
    })
}

pub fn init(accounts: Vec<String>) {
    let phone_number_hashes = accounts.iter()
        .map(|x| blake3::hash(x.as_bytes()));

    let phone_number_hash_set = HashSet::from_iter(phone_number_hashes);

    PHONE_NUMBER_INDEX.with(|index| {
        index.replace(phone_number_hash_set);
    })
}