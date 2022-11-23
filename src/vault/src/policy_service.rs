use std::borrow::Borrow;
use std::collections::HashMap;

use candid::{candid_method, CandidType, Principal};
use ic_cdk::{caller, storage, trap};
use ic_ledger_types::Tokens;
use serde::{Deserialize, Serialize};
use crate::{POLICIES, Transaction};


#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Policy {
    pub id: u64,
    pub amount_threshold: u8,
    pub currency: Currency,
    pub member_threshold: u8,
    pub accounts: Vec<u64>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum Currency {
    ICP,
}

pub fn register_policy() -> Policy {
    POLICIES.with(|policies| {
        let ps = Policy {
            id: (policies.borrow().len() + 1) as u64,
            amount_threshold: 0,
            currency: Currency::ICP,
            accounts: vec![],
            member_threshold: 0,
        };
        policies.borrow_mut().insert(ps.id, ps.clone());
        ps
    })
}

pub fn is_passed(transaction: Transaction) -> bool { //TODO
    // let ps = storage::get_mut::<Policies>().get(&policy_id).unwrap();
    // return ps.sum_threshold <= sum_threshold && ps.member_threshold <= member_threshold
    true
}