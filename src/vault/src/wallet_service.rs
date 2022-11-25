use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;

use candid::{CandidType, Principal};
use ic_cdk::trap;
use ic_ledger_types::{AccountIdentifier, Subaccount};
use serde::Deserialize;

use crate::WALLETS;

pub type Wallets = HashMap<u64, Wallet>;
pub type AccountIdConstraint = u64; //todo

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Wallet {
    pub id: u64,
    pub name: Option<String>,
    pub vault_ids: Vec<u64>,
    //todo move to index?
    // pub transaction_ids: Vec<u64>, //todo move to index?
}

pub fn new_and_store(name: Option<String>, vault_id: u64) -> Wallet {
    WALLETS.with(|wallets| {
        let mut w = wallets.borrow_mut();
        let id = w.len() as u64;
        let wlt = Wallet {
            id: id.clone(),
            name,
            vault_ids: vec![vault_id],
            // transaction_ids: vec![]
        };
        w.insert(id, wlt.clone());
        wlt
    })
}

pub fn restore(wallet: Wallet) -> Option<Wallet> {
    return WALLETS.with(|wallets| {
        wallets.borrow_mut().insert(wallet.id, wallet.clone())
    });
}

pub fn get_wallet(id: u64) -> Wallet {
    WALLETS.with(|wallets| {
        match wallets.borrow().get(&id) {
            None => {
                trap("Not registered")
            }
            Some(wallet) => {
                wallet.clone()
            }
        }
    })
}


pub fn id_to_subaccount(id: u64) -> Subaccount {
    let eights: [u8; 8] = bytemuck::cast([id; 1]);
    let mut whole: [u8; 32] = [0; 32];
    let (one, two) = whole.split_at_mut(8);
    one.copy_from_slice(&eights); //todo think about it!
    return Subaccount(whole);
}

pub fn id_to_address(id: u64) -> AccountIdentifier {
    let eights: [u8; 8] = bytemuck::cast([id; 1]);
    let mut whole: [u8; 32] = [0; 32];
    let (one, two) = whole.split_at_mut(8);
    one.copy_from_slice(&eights); //todo think about it!
    return AccountIdentifier::new(&Principal::management_canister(), &Subaccount(whole));
}

