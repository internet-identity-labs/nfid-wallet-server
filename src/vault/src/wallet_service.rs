use std::borrow::{Borrow, BorrowMut};

use candid::{CandidType, Principal};
use ic_cdk::{id, trap};
use ic_ledger_types::{AccountIdentifier, Subaccount};
use serde::Deserialize;

use crate::WALLETS;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Wallet {
    pub id: u64,
    pub name: Option<String>,
    pub vaults: Vec<u64>,
}

pub fn new_and_store(name: Option<String>, vault_id: u64) -> Wallet {
    WALLETS.with(|wallets| {
        let mut w = wallets.borrow_mut();
        let id = w.len() as u64 + 1;
        let wlt = Wallet {
            id: id.clone(),
            name,
            vaults: vec![vault_id],
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

pub fn get_wallets(ids: Vec<u64>) -> Vec<Wallet> {
    WALLETS.with(|wallets| {
        let mut result: Vec<Wallet> = Default::default();
        for key in ids {
            match wallets.borrow().get(&key) {
                None => {
                    trap("Not registered")
                }
                Some(wallet) => {
                    result.push(wallet.clone())
                }
            }
        }
        result
    })
}

pub fn id_to_subaccount(id: u64) -> Subaccount {
    let eights: [u8; 8] = bytemuck::cast([id; 1]);
    let mut whole: [u8; 32] = [0; 32];
    let (one, two) = whole.split_at_mut(8);
    one.copy_from_slice(&eights);
    return Subaccount(whole);
}

pub fn id_to_address(wallet_id: u64) -> AccountIdentifier {
    let mut whole: Subaccount =id_to_subaccount(wallet_id);
    return AccountIdentifier::new(&id(), &(whole));
}


