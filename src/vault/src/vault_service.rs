use candid::CandidType;
use ic_cdk::{caller, trap};
use serde::Deserialize;

use crate::{user_service, VAULTS};
use crate::VaultRole::VaultOwner;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Vault {
    pub id: u64,
    pub name: String,
    pub wallets: Vec<u64>,
    pub policies: Vec<u64>,
    pub participants: Vec<VaultMember>,
}

#[derive(Clone, Debug, CandidType, Deserialize, Copy)]
pub struct VaultMember {
    pub user_id: u64,
    pub role: VaultRole,
}

#[derive(Clone, Debug, CandidType, Deserialize, Copy)]
pub enum  VaultRole {
    VaultOwner,
    VaultApprove,
}


impl PartialEq for Vault {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}


pub fn register(name: String) -> Vault {
    VAULTS.with(|vaults| {
        let vault_id = (vaults.borrow().len() + 1) as u64;
        let address = caller().to_text();

        let user = user_service::get_or_new_by_address(address, vault_id);

        let participants: Vec<VaultMember> = vec![VaultMember { user_id: user.id, role: VaultOwner }];

        let g: Vault = Vault {
            id: vault_id,
            name,
            wallets: vec![],
            policies: vec![],
            participants,
        };
        vaults.borrow_mut().insert(vault_id, g.clone());
        return g;
    })
}

pub fn get_by_ids(ids: Vec<u64>) -> Vec<Vault> {
    VAULTS.with(|vaults| {
        let mut result: Vec<Vault> = Default::default();
        for key in ids {
            match vaults.borrow_mut().get(&key) {
                None => {
                    trap("Nonexistent key error")
                }
                Some(v) => { result.push(v.clone()) }
            }
        }
        result
    })
}

pub fn restore(vault: Vault) -> bool {
    VAULTS.with(|vaults| {
        vaults.borrow_mut().insert(vault.id, vault).is_some()
    })
}
