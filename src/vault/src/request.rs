use ic_cdk::export::{candid::{CandidType, Deserialize}};

use crate::{PolicyType, State, VaultRole};

#[derive(CandidType, Deserialize, Clone)]
pub struct TransactionRegisterRequest {
    pub amount: u64,
    pub address: String,
    pub wallet_id: u64,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct VaultRegisterRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct VaultMemberRequest {
    pub vault_id: u64,
    pub address: String,
    pub name: Option<String>,
    pub role: VaultRole,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct WalletRegisterRequest {
    pub vault_id: u64,
    pub name: Option<String>,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct TransactionApproveRequest {
    pub transaction_id: u64,
    pub state: State,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct PolicyRegisterRequest {
   pub vault_id: u64,
   pub policy_type: PolicyType,
}