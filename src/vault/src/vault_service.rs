use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::hash::{Hash, Hasher};
use ic_cdk::trap;
use ic_cdk::export::{candid::{CandidType, Deserialize}};

use crate::VAULTS;
use crate::VaultRole::Admin;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Vault {
    pub id: u64,
    pub name: String,
    pub wallets: Vec<u64>,
    pub policies: Vec<u64>,
    pub members: HashSet<VaultMember>,
}

#[derive(Clone, Debug, CandidType, Deserialize, Eq, PartialEq)]
pub struct VaultMember {
    pub user_uuid: String,
    pub role: VaultRole,
    pub name: Option<String>,
}

impl Hash for VaultMember {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.user_uuid.hash(state)
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Copy, Eq, PartialEq)]
pub enum VaultRole {
    Admin,
    Member,
}


impl PartialEq for Vault {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}


pub fn register(user_uuid: String, name: String) -> Vault {
    VAULTS.with(|vaults| {
        let vault_id = (vaults.borrow().len() + 1) as u64;
        let mut participants: HashSet<VaultMember> = Default::default();
        let owner = VaultMember { user_uuid: user_uuid, role: Admin, name: None };
        participants.insert(owner);
        let g: Vault = Vault {
            id: vault_id,
            name,
            wallets: vec![],
            policies: vec![],
            members: participants,
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
