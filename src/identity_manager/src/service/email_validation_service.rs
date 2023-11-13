use crate::structure::ttl_hashmap::TtlHashMap;
use std::cell::RefCell;

thread_local! {
    static TOKENS_REPOSITORY: RefCell<TtlHashMap<String, String>> = RefCell::new(TtlHashMap::new(900000));
}

pub fn insert(key: String, value: String, timestamp: u64) -> () {
    TOKENS_REPOSITORY.with(|repository| {
        let mut repo = repository.borrow_mut();
        repo.clean_expired_entries(timestamp);
        repo.insert(key, value, timestamp);
    });
} 

pub fn contains(key: String, value: String) -> bool {
    TOKENS_REPOSITORY.with(|repository| {
        match repository.borrow().get(&key) {
            Some(val) => value.eq(val),
            None => false,
        }
    })
}
