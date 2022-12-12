use candid::CandidType;
use ic_cdk::trap;
use serde::Deserialize;

use crate::POLICIES;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Policy {
    pub id: u64,
    pub policy_type: PolicyType,
    pub created_date: u64,
    pub modified_date: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum PolicyType {
    #[serde(rename = "threshold_policy")]
    ThresholdPolicy(ThresholdPolicy)
}


#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ThresholdPolicy {
    pub amount_threshold: u64,
    pub currency: Currency,
    pub member_threshold: u8,
    pub wallet_ids: Option<Vec<u64>>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum Currency {
    ICP,
}

pub fn register_policy(policy_type: PolicyType) -> Policy {
    POLICIES.with(|policies| {
        let ps = Policy {
            id: (policies.borrow().len() + 1) as u64,
            policy_type,
            created_date: ic_cdk::api::time(),
            modified_date: ic_cdk::api::time(),
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

pub fn get(ids: Vec<u64>) -> Vec<Policy> {
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

pub fn define_correct_policy(ids: Vec<u64>, amount: u64, wallet_id: u64) -> Policy {
    get(ids).into_iter()
        .map(|l| match l.policy_type.clone() {
            PolicyType::ThresholdPolicy(threshold_policy) => {
                match threshold_policy.wallet_ids {
                    Some(x) => {
                        if x.contains(&wallet_id)
                        { Some((l, threshold_policy.amount_threshold)) } else { None }
                    }
                    None => {
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
        .map(|l| l.0)
        .unwrap()
}