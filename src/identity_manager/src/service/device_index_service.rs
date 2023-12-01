use std::{cell::RefCell, collections::VecDeque, convert::TryInto};

use ic_cdk::{storage, api::set_certified_data};
use ic_certified_map::AsHashTree;

use crate::repository::account_repo::PrincipalIndex;
use super::certified_service::TREE;

thread_local! {
    pub static DEVICE_INDEX_STACK: RefCell<VecDeque<(String, Vec<u8>)>> = RefCell::new(VecDeque::new());
}

pub fn save_temp_stack() -> String {
    let is_empty = DEVICE_INDEX_STACK.with(|index_ref| {
        index_ref.borrow().is_empty()
    });

    if !is_empty {
        return String::from("The stack is not empty. No action required.")
    }

    let device_index_iterator = storage::get::<PrincipalIndex>()
        .into_iter()
        .map(|(device, root)| {
            let root_hex = hex::decode(sha256::digest(root.clone())).unwrap();
            (device.clone(), root_hex) 
        });

    DEVICE_INDEX_STACK.with(|index| {
        index.borrow_mut().extend(device_index_iterator);
    });

    String::from("The stack has been filled with data.")
}

pub fn get_remaining_size_after_rebuild_index_slice_from_temp_stack(amount_opt: Option<u64>) -> u64 {
    let (slice, remaining_size): (Vec<(String, Vec<u8>)>, usize) = DEVICE_INDEX_STACK.with(|index_ref| {
        let mut index = index_ref.borrow_mut();
        let mut amount: usize = amount_opt.map_or_else(|| 10_000, |v| v.try_into().unwrap());
        amount = amount.min(index.len());
        let slice = index.drain(..amount).collect();
        let index_len = index.len();
        (slice, index_len)
    });

    if slice.is_empty() {
        return 0;
    }

    TREE.with(|keys_ref| {
        let mut keys = keys_ref.borrow_mut();

        for (device, root_hex) in slice {
            keys.insert(device, root_hex);
        }

        set_certified_data(&keys.root_hash());
    });

    remaining_size.try_into().unwrap()
}