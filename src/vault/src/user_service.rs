use candid::CandidType;
use ic_cdk::trap;
use serde::Deserialize;

use crate::USERS;
use crate::util::caller_to_address;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct User {
    pub address: String,
    pub vaults: Vec<u64>,
}

pub fn get_or_new_by_address(address: String) -> User {
    USERS.with(|users| {
        let mut borrowed = users.borrow_mut();
        match borrowed.get_mut(&address) {
            None => {
                let p = User { address: address.clone(), vaults: vec![] };
                borrowed.insert(address, p.clone());
                p
            }
            Some(u) => {
                u.clone()
            }
        }
    })
}


pub fn get_by_address(address: String) -> User {
    USERS.with(|users| {
        match users.borrow_mut().get_mut(&address) {
            None => {
                trap("Not registered")
            }

            Some(p) => {
                p.clone()
            }
        }
    })
}

pub fn restore(user: User) -> Option<User> {
    USERS.with(|users| {
        users.borrow_mut().insert(user.address.clone(), user)
    })
}


pub fn get_by_caller() -> User {
    let address = caller_to_address();
    get_by_address(address)
}
