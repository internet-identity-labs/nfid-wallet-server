use std::borrow::{Borrow, BorrowMut};
use std::collections::{HashMap, HashSet};

use candid::CandidType;
use ic_cdk::trap;
use ic_ledger_types::BlockIndex;
use serde::{Deserialize, Serialize};

use crate::{caller_to_address, Policy, PolicyType, TRANSACTIONS, User};
use crate::enums::State;
use crate::policy_service::Currency;

pub type Transactions = HashMap<u64, Transaction>;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Transaction {
    pub id: u64,
    pub wallet_id: u64,
    pub vault_id: u64,
    pub to: String,
    pub approves: HashSet<Approve>,
    pub amount: u64,
    pub state: State,
    pub policy_id: u64,
    pub block_index: Option<BlockIndex>,
    pub amount_threshold: u64,
    pub currency: Currency,
    pub member_threshold: u8,
    pub created_date: u64,
    pub modified_date: u64,
}


#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Hash, Eq)]
pub struct Approve {
    pub signer: String,
    pub created_date: u64,
    pub status: State,
}

impl PartialEq for Approve {
    fn eq(&self, other: &Self) -> bool {
        self.signer.eq(&other.signer)
    }
}

pub fn register_transaction(amount: u64, to: String, wallet_id: u64, policy: Policy, vault_id: u64) -> Transaction {
    let amount_threshold: u64;
    let member_threshold: u8;

    match policy.policy_type.clone() {
        PolicyType::ThresholdPolicy(tp) => {
            amount_threshold = tp.amount_threshold;
            member_threshold = tp.member_threshold;
        }
    }
    TRANSACTIONS.with(|transactions| {
        let mut ts = transactions.borrow_mut();
        let mut approves: HashSet<Approve> = Default::default();
        let approve = Approve {
            signer: caller_to_address(),
            created_date: ic_cdk::api::time(),
            status: State::APPROVED,
        };
        approves.insert(approve);
        let t: Transaction = Transaction {
            id: (ts.len() + 1) as u64,
            wallet_id,
            vault_id,
            to,
            approves,
            amount,
            state: State::PENDING,
            policy_id: policy.id,
            block_index: None,
            amount_threshold,
            currency: Currency::ICP,
            member_threshold,
            created_date: ic_cdk::api::time(),
            modified_date: ic_cdk::api::time(),
        };
        ts.insert(t.id, t.clone());
        t
    })
}


pub fn approve_transaction(transaction_id: u64, signer: User, state: State) -> Transaction {
    TRANSACTIONS.with(|transactions| {
        match transactions.borrow_mut().get_mut(&transaction_id) {
            None => {
                trap("Not registered")
            }
            Some(ts) => {
                ts.approves.insert(
                    Approve {
                        signer: signer.address,
                        created_date: ic_cdk::api::time(),
                        status: state,
                    });
                ts.modified_date = ic_cdk::api::time();
                ts.clone()
            }
        }
    })
}

pub fn store_transaction(transaction: Transaction) -> Option<Transaction> {
    TRANSACTIONS.with(|transactions| {
        return transactions.borrow_mut().insert(transaction.id, transaction);
    })
}

pub fn get_all(vaults: Vec<u64>) -> Vec<Transaction> {
    TRANSACTIONS.with(|transactions| {
        return transactions.borrow().iter()
            .map(|a| a.1.clone())
            .filter(|t| vaults.contains(&t.vault_id))
            .collect();
    })
}
