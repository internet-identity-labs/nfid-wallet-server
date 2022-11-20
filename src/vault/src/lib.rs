extern crate core;

use std::borrow::Borrow;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::ops::Sub;

use candid::{candid_method, CandidType, Principal};
use ic_cdk::{caller, storage, trap};
use ic_cdk_macros::*;
use ic_ledger_types::{AccountIdentifier, BlockIndex, DEFAULT_FEE, DEFAULT_SUBACCOUNT, MAINNET_LEDGER_CANISTER_ID, Memo, Subaccount, Tokens};
use serde::{Deserialize, Serialize};

pub type Groups = HashMap<u64, Group>;
pub type Transactions = HashMap<u64, Transaction>;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Group {
    pub group_id: u64,
    pub name: String,
    pub accounts: Vec<Account>,
    pub participants: Vec<String>,
    pub threshold: u8,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Account {
    pub account_id: u8,
    pub name: String,
    pub sub_account: Subaccount,
    // pub polici: Policy (list Rules)
}



#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Participant {
    principal: String,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Transaction {
    pub id: u64,
    pub group_id: u64,
    pub to: AccountIdentifier,
    pub approves: HashSet<String>,
    pub initiator: u8,
    pub amount: Tokens,

    // pub expires //todo
}


impl PartialEq for Group {
    fn eq(&self, other: &Self) -> bool {
        self.group_id.eq(&other.group_id)
    }
}


#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Hash)]
pub struct TransferArgs {
    amount: Tokens,
    to_principal: Principal,
    to_subaccount: Option<Subaccount>,
}


#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Hash, PartialEq)]
pub struct Conf {
    ledger_canister_id: Principal,
}


impl Default for Conf {
    fn default() -> Self {
        Conf {
            ledger_canister_id: MAINNET_LEDGER_CANISTER_ID,
        }
    }
}

thread_local! {
    static CONF: RefCell<Conf> = RefCell::new(Conf::default());
}
#[init]
#[candid_method(init)]
fn init(conf: Conf) {
    CONF.with(|c| c.replace(conf));
}

#[query]
async fn sub(princ: String, group: u64, acc: u8) -> String {
    let sixty_fours: [u64; 4] = [group; 4];
    let mut eights: [u8; 32] = bytemuck::cast(sixty_fours);
    eights[31] = eights[31] + acc;
    let to_subaccount = Subaccount(eights);
    AccountIdentifier::new(Principal::from_text(princ).unwrap().borrow(), &to_subaccount).to_string()
}

#[update]
async fn register_group(name: String) {
    let caller = caller().to_text();
    let mut  participants: Vec<String> = Default::default();
    participants.push(caller); //todo role + sub_account
    let groups = storage::get_mut::<Groups>();
    let group_id = (groups.keys().count() + 1) as u64;
    let sixty_fours: [u64; 4] = [group_id; 4];
    let mut eights: [u8; 32] = bytemuck::cast(sixty_fours);
    eights[31] = eights[31] + 1;

    let default = Account {
        account_id: 1,
        name : "Account 1".to_string(),
        sub_account: Subaccount(eights)
    };

    let accounts = vec![default];

    let g: Group = Group {
        group_id,
        name,
        accounts,
        participants,
        threshold:  1 ,
    };
    storage::get_mut::<Groups>().insert(group_id, g);
}


#[update]
async fn register_transaction(owner: u8, amount: Tokens, to: Principal, sub: Option<Subaccount>, group_id: u64) {
    let mut g = storage::get_mut::<Groups>().get_mut(group_id.borrow()).unwrap();

    let caller = g.participants.clone().into_iter() //todo
        .find(|l| l.eq(&caller().to_text()));

    if caller.is_none() {
        trap("Fuck you! Cheater!")
    }

    let to = AccountIdentifier::new(to.borrow(), &sub.unwrap_or(DEFAULT_SUBACCOUNT));

    let mut approves: HashSet<String> = Default::default();
    approves.insert(caller.unwrap());

    let mut ts = storage::get_mut::<Transactions>();

    let t: Transaction = Transaction {
        id: ts.len() as u64,
        group_id,
        to,
        approves,
        initiator: owner,
        amount,
    };

    ts.insert(t.id, t);
}


#[update]
async fn add_participant(group_id: u64, principal: String) {
    let mut g = storage::get_mut::<Groups>().get_mut(group_id.borrow()).unwrap();
    let caller = g.participants.clone().into_iter().find(|l| l.eq(&caller().to_text()));
    if caller.is_none() {
        trap("Fuck you! Cheater!")
    }
    g.participants.push(principal);
}


#[update]
async fn approve_transaction(transaction_id: u64) {
    let mut tss = storage::get_mut::<Transactions>().get_mut(&transaction_id);


    if tss.is_none() {
        trap("No such ts.")
    }

    let ts = tss.unwrap();

    let mut g = storage::get_mut::<Groups>().get_mut(ts.group_id.borrow()).unwrap();

    let caller = g.participants.clone().into_iter().find(|l| l.eq(&caller().to_text()));

    if caller.is_none() {
        trap("Fuck you! Cheater!")
    }

    let threshold = g.threshold;

    ts.approves.insert(caller.unwrap());

    let len = ts.approves.len() as u8;
    if threshold == len {
        transfer(ts.amount, ts.to, g.accounts[0].sub_account).await;
    }
}


async fn transfer(amount: Tokens, to: AccountIdentifier, from_subaccount: Subaccount) -> Result<BlockIndex, String> {
    let ledger_canister_id = MAINNET_LEDGER_CANISTER_ID;
    let transfer_args = ic_ledger_types::TransferArgs {
        memo: Memo(0),
        amount,
        fee: DEFAULT_FEE,
        from_subaccount: Some(from_subaccount),
        to,
        created_at_time: None,
    };
    ic_ledger_types::transfer(ledger_canister_id, transfer_args).await
        .map_err(|e| format!("failed to call ledger: {:?}", e))?
        .map_err(|e| format!("ledger transfer error {:?}", e))
}

#[test]
fn sub_account_test() {
    let group_id = 2;

    let sixty_fours: [u64; 4] = [group_id; 4];
    let mut eights: [u8; 32] = bytemuck::cast(sixty_fours);
    let mut nines: [u8; 32] = bytemuck::cast(sixty_fours);
    let s: [u64; 4] =  bytemuck::cast(eights);
    assert!(eights.eq(&nines));
    assert!(s.eq(&sixty_fours))
}


