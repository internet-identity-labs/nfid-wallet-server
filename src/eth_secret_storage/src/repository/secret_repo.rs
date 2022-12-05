use std::cell::RefCell;
use std::collections::HashMap;

use crate::constant::constant::Application::Metamask;

thread_local! {
    static SECRET_STORAGE: RefCell<HashMap<String, HashMap<String, String>>> = RefCell::new(HashMap::from([(Metamask.value(), HashMap::new())]));
}

pub fn get(app: String, address: String) -> Option<Option<String>> {
    SECRET_STORAGE.with(|storage| {
        storage.borrow().get(&app).map(|map| map.get(&address).map(|s| s.clone()))
    })
}

pub fn save(app: String, address: String, secret: String) -> Option<Option<String>> {
    SECRET_STORAGE.with(|storage| {
        let mut storage_mut = storage.borrow_mut();
        storage_mut.get_mut(&app).map(|map| {
            match map.get(&address) {
                Some(secret) => Some(secret.clone()),
                None => {
                    map.insert(address.clone(), secret.clone());
                    Some(secret)
                }
            }
        })
    })
}

pub fn get_all() -> HashMap<String, HashMap<String, String>> {
    SECRET_STORAGE.with(|storage| {
        storage.borrow().to_owned()
    })
}

pub fn save_all(map: HashMap<String, HashMap<String, String>>) -> () {
    SECRET_STORAGE.with(|storage| {
        storage.borrow_mut().extend(map.into_iter())
    })
}