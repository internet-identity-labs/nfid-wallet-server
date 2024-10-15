use crate::structure::ttl_hashmap::TtlHashMap;
use crate::{HttpResponse, to_success_response};
use std::cell::RefCell;
use ic_cdk::trap;

thread_local! {
    static TOKENS_REPOSITORY: RefCell<TtlHashMap<String, String>> = RefCell::new(TtlHashMap::new(900000));
}

pub fn insert(key: String, value: String, timestamp: u64) -> HttpResponse<bool> {
    if !is_valid_email_address_size(&key) {
        trap("Incorrect email address size: it's more than 320 characters.");
    }

    TOKENS_REPOSITORY.with(|repository| {
        let mut repo = repository.borrow_mut();
        repo.clean_expired_entries(timestamp);
        repo.insert(key, value, timestamp);
    });

    to_success_response(true)
} 

pub fn contains(key: String, value: String) -> bool {
    TOKENS_REPOSITORY.with(|repository| {
        match repository.borrow().get(&key) {
            Some(val) => value.eq(val),
            None => false,
        }
    })
}

fn is_valid_email_address_size(email: &str) -> bool {
    email.len() <= 320
}