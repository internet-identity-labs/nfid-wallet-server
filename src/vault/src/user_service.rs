use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;

use candid::{candid_method, CandidType, Principal};
use ic_cdk::{caller, storage, trap};
use serde::{Deserialize, Serialize};

use crate::USERS;

// pub type Users = HashMap<String, User>;


#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct User {
    pub id: u64,
    pub address: String,
    pub vaults: Vec<u64>,
}

pub fn get_or_new_by_address(address: String, group_id: u64) -> User {
    USERS.with(|users| {
        match users.borrow_mut().get_mut(&address) {
            None => {
                let id = users.borrow().keys().len() + 1;
                let p = User { id: id as u64, address: address.clone(), vaults: vec![group_id] };
                users.borrow_mut().insert(address, p.clone());
                p
            }
            Some(u) => {
                u.vaults.push(group_id);
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
