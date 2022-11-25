use std::borrow::Borrow;

use candid::CandidType;
use ic_cdk::trap;
use serde::Deserialize;

use crate::{POLICIES, Transaction};

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Policy {
    pub id: u64,
    pub policy_class: PolicyClass,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum PolicyClass {
    #[serde(rename = "threshold")]
    ThresholdPolicy(ThresholdPolicy)
}


#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ThresholdPolicy {
    pub amount_threshold: u64,
    pub currency: Currency,
    pub member_threshold: u8,
    pub sub_class: ThresholdPolicySubClass,
}


#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum ThresholdPolicySubClass {
    #[serde(rename = "wallet_specific")]
    WalletSpecific(WalletSpecific),
    #[serde(rename = "wallet_common")]
    WalletCommon,
}


#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct WalletSpecific {
    pub wallet_ids: Vec<u64>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum Currency {
    ICP,
}

pub fn register_policy(amount_threshold: u64, member_threshold: u8, wallet_ids: Vec<u64>) -> Policy {
    let policy_sub_class = if wallet_ids.is_empty()
    { ThresholdPolicySubClass::WalletCommon } else { ThresholdPolicySubClass::WalletSpecific(WalletSpecific { wallet_ids }) };

    let threshold_policy = ThresholdPolicy {
        amount_threshold,
        currency: Currency::ICP,
        member_threshold,
        sub_class: policy_sub_class,
    };

    POLICIES.with(|policies| {
        let ps = Policy {
            id: (policies.borrow().len() + 1) as u64,
            policy_class: PolicyClass::ThresholdPolicy(threshold_policy),
        };
        policies.borrow_mut().insert(ps.id, ps.clone());
        ps
    })
}

pub fn restore_policy(ps: Policy) -> Policy {
    POLICIES.with(|policies| {
        policies.borrow_mut().insert(ps.id, ps.clone());
        ps
    })
}

pub fn get_by_ids(ids: Vec<u64>) -> Vec<Policy> {
    POLICIES.with(|policies| {
        let mut result: Vec<Policy> = Default::default();
        for id in ids {
            match policies.borrow_mut().get(&id) {
                None => {
                    trap("Nonexistent key error")
                }
                Some(v) => { result.push(v.clone()) }
            }
        }
        result
    })
}

pub fn define_correct_policy(policies: Vec<Policy>, amount: u64, wallet_id: u64) -> Policy {
    policies.into_iter()
        .map(|l| match l.policy_class.clone() {
            PolicyClass::ThresholdPolicy(threshold_policy) => {
                match threshold_policy.sub_class {
                    ThresholdPolicySubClass::WalletSpecific(x) => {
                        if x.wallet_ids.contains(&wallet_id)
                        { Some((l, threshold_policy.amount_threshold)) } else { None } //TODO шото мне оно не нравится
                    }
                    ThresholdPolicySubClass::WalletCommon => {
                        Some((l, threshold_policy.amount_threshold))
                    }
                }
            }
        }
        )
        .filter(|l| l.is_some())
        .map(|l| l.unwrap())
        .filter(|l| l.1 <= amount)
        .reduce(|a, b| if a.1 > b.1 { a } else { b })
        .map(|l|l.0)
        .unwrap()
}