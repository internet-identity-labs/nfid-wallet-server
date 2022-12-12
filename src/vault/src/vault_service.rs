use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use ic_cdk::export::{candid::{CandidType, Deserialize}};
use ic_cdk::trap;
use crate::enums::ObjectState;
use crate::memory::VAULTS;
use crate::User;

use crate::VaultRole::Admin;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Vault {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
    pub wallets: HashSet<u64>,
    pub policies: HashSet<u64>,
    pub members: HashSet<VaultMember>,
    pub state: ObjectState,
    pub created_date: u64,
    pub modified_date: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize, Eq, PartialEq)]
pub struct VaultMember {
    pub user_uuid: String,
    pub role: VaultRole,
    pub name: Option<String>,
    pub state: ObjectState,
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


pub fn register(user_uuid: String, name: String, description: Option<String>) -> Vault {
    VAULTS.with(|vaults| {
        let vault_id = (vaults.borrow().len() + 1) as u64;
        let mut participants: HashSet<VaultMember> = Default::default();
        let owner = VaultMember { user_uuid, role: Admin, name: None, state: ObjectState::Active };
        participants.insert(owner);
        let vault_new: Vault = Vault {
            id: vault_id,
            name,
            description,
            wallets: hashset![],
            policies: hashset![],
            members: participants,
            state: ObjectState::Active,
            created_date: ic_cdk::api::time(),
            modified_date: ic_cdk::api::time(),
        };
        vaults.borrow_mut().insert(vault_id, vault_new.clone());
        return vault_new;
    })
}

pub fn get(ids: HashSet<u64>) -> Vec<Vault> {
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

pub fn get_by_id(id: u64) -> Vault {
    VAULTS.with(|vaults| {
        match vaults.borrow_mut().get(&id) {
            None => {
                trap("Nonexistent key error")
            }
            Some(v) => {
                v.clone()
            }
        }
    })
}

pub fn add_vault_member(vault_id: u64, user: &User, role: VaultRole, name:Option<String>) -> Vault {
    let mut vault = get_by_id(vault_id);
    let vm = VaultMember {
        user_uuid: user.address.clone(),
        role,
        name,
        state: ObjectState::Archived
    };
    vault.members.insert(vm);
    restore(vault)
}

pub fn restore(mut vault: Vault) -> Vault {
    VAULTS.with(|vaults| {
        vault.modified_date = ic_cdk::api::time();
        match vaults.borrow_mut().insert(vault.id, vault.clone()) {
            None => {
                trap("No such vault")
            }
            Some(_) => {
                vault
            }
        }
    })
}
