use std::borrow::{Borrow, BorrowMut};
use std::collections::{HashMap, HashSet};

use candid::{candid_method, CandidType, Principal};
use ic_cdk::{caller, storage, trap};
use ic_ledger_types::{AccountIdentifier, BlockIndex, Subaccount, Tokens};
use serde::{Deserialize, Serialize};

use crate::{Policy, TRANSACTIONS, transfer, User, user_service, wallet_service};
use crate::policy_service::{Currency};

pub type Transactions = HashMap<u64, Transaction>;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Transaction {
    pub id: u64,
    pub wallet_id: u64,
    pub to: AccountIdentifier,
    pub approves: HashSet<Approve>,
    pub amount: Tokens,
    pub active: bool,
    pub policy_id: u64,
    pub block_index: Option<BlockIndex>,
    pub amount_threshold: u64,
    pub currency: Currency,
    pub member_threshold: u8,
}


#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Hash, Eq)]
pub struct Approve {
    signer: String,
    created_date: u64,

}

impl PartialEq for Approve {
    fn eq(&self, other: &Self) -> bool {
        self.signer.eq(&other.signer)
    }
}

pub fn register_transaction(amount: Tokens, to: AccountIdentifier, wallet_id: u64, tr_owner: User, policy: Policy) -> Transaction {
    TRANSACTIONS.with(|transactions| {
        let mut ts = transactions.borrow_mut();
        let mut approves: HashSet<Approve> = Default::default();
        let approve = Approve {
            signer: tr_owner.address,
            created_date: ic_cdk::api::time(),
        };
        approves.insert(approve);
        let t: Transaction = Transaction {
            id: ts.len() as u64,
            wallet_id,
            to,
            approves,
            amount,
            active: true,
            policy_id: policy.id, 
            block_index: None,
            amount_threshold: 0,
            currency: Currency::ICP,
            member_threshold: 0
        };
        ts.insert(t.id, t.clone());
        t
    })
}


pub fn approve_transaction(transaction_id: u64, signer: User) -> Transaction {
    TRANSACTIONS.with(|transactions| {
        match transactions.borrow_mut().get_mut(&transaction_id) {
            None => {
                trap("No such ts.")
            }
            Some(tss) => {
                tss.approves.insert(Approve { signer: signer.address, created_date: ic_cdk::api::time() });
                return tss.clone();
            }
        }
    })
}

pub fn store_transaction(transaction: Transaction) -> Option<Transaction> {
    TRANSACTIONS.with(|transactions| {
        return transactions.borrow_mut().insert(transaction.id, transaction);
    })
}