use candid::CandidType;
use ic_cdk::{call, caller, trap};
use serde::Deserialize;

use crate::USERS;



#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct User {
    pub address: String,
    pub vaults: Vec<u64>,
}

pub fn get_or_new_by_address(address: String, vault_id: u64) -> User {
    USERS.with(|users| {
        match users.borrow_mut().get_mut(&address) {
            None => {
                let p = User { address: address.clone(), vaults: vec![vault_id] };
                users.borrow_mut().insert(address, p.clone());
                p
            }
            Some(u) => {
                u.vaults.push(vault_id);
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


pub fn get_by_caller() -> User {
    let address = caller();
    USERS.with(|users| {
        match users.borrow_mut().get_mut(&address.to_text()) {
            None => {
                trap("Not registered")
            }

            Some(p) => {
                p.clone()
            }
        }
    })
}
