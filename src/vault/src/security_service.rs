use ic_cdk::trap;

use crate::{get_or_new_by_caller, vault_service, VaultRole};

pub fn trap_if_not_permitted(vault_id: u64, accepted_roles: Vec<VaultRole>) {
    let caller = get_or_new_by_caller();
    let vault = vault_service::get_by_id(vault_id);
    let caller_member = vault.members
        .iter()
        .find(|p| caller.address.eq(&p.user_uuid));
    match caller_member {
        None => {
            trap("Unauthorised")
        }
        Some(vault_member) => {
            if !accepted_roles.is_empty() && !accepted_roles.contains(&vault_member.role) {
                trap("Not enough permissions")
            }
        }
    }
}
