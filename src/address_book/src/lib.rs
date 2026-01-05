mod types;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use candid::Principal;
use ic_cdk::{call, caller, id, storage};
use ic_cdk::api::call::CallResult;
use ic_cdk::api::management_canister::main::CanisterStatusResponse;
use ic_cdk_macros::*;

pub use types::{Address, AddressBookError, AddressType, CanisterIdRequest, Conf, Memory, User, UserAddress};

thread_local! {
    static CONFIG: RefCell<Conf> = const { RefCell::new(Conf {
        max_user_addresses: 50,
        max_name_length: 200,
    }) };
    static ADDRESS_BOOK: RefCell<HashMap<String, User>> = RefCell::new(HashMap::default());
}

#[pre_upgrade]
pub fn stable_save() {
    let data = ADDRESS_BOOK.with(|book| book.borrow().clone());
    let config = CONFIG.with(|c| c.borrow().clone());

    let mem = Memory {
        data,
        config,
    };

    storage::stable_save((mem,))
        .expect("Stable save exited unexpectedly: unable to save data to stable memory.");
}

#[post_upgrade]
pub fn stable_restore() {
    let (mem,): (Memory,) = storage::stable_restore()
        .expect("Stable restore exited unexpectedly: unable to restore data from stable memory.");

    let Memory { config, data } = mem;

    CONFIG.with(|c| {
        *c.borrow_mut() = config;
    });

    ADDRESS_BOOK.with(|book| {
        *book.borrow_mut() = data;
    });
}

#[update]
pub async fn save(user_address: UserAddress) -> Result<Vec<UserAddress>, AddressBookError> {
    let caller = caller().to_text();

    let (max_name_length, max_addresses) = CONFIG.with(|c| {
        let conf = c.borrow();
        (conf.max_name_length, conf.max_user_addresses)
    });

    if user_address.name.len() > max_name_length as usize {
        return Err(AddressBookError::NameTooLong);
    }

    ADDRESS_BOOK.with(|book| {
        let mut book = book.borrow_mut();
        let user = book.entry(caller.clone()).or_insert_with(|| User {
            user_addresses: HashSet::new(),
        });

        validate_no_duplicate_names(&user.user_addresses, &user_address)?;
        validate_no_duplicate_addresses(&user.user_addresses, &user_address)?;

        let is_new = !user.user_addresses.contains(&user_address);

        if is_new {
            if user.user_addresses.len() >= max_addresses as usize {
                return Err(AddressBookError::MaxAddressesReached);
            }
        }

        user.user_addresses.replace(user_address);

        let addresses = get_user_addresses(&book, &caller);
        Ok(addresses)
    })
}

#[update]
pub async fn delete(id: String) -> Result<Vec<UserAddress>, AddressBookError> {
    let caller = caller().to_text();

    ADDRESS_BOOK.with(|book| {
        let mut book = book.borrow_mut();

        if let Some(user) = book.get_mut(&caller) {
            let temp_address = UserAddress {
                id: id.clone(),
                name: String::new(),
                addresses: Vec::new(),
            };

            if user.user_addresses.remove(&temp_address) {
                let addresses = get_user_addresses(&book, &caller);
                Ok(addresses)
            } else {
                Err(AddressBookError::AddressNotFound)
            }
        } else {
            Err(AddressBookError::AddressNotFound)
        }
    })
}

#[query]
pub fn find_all() -> Result<Vec<UserAddress>, AddressBookError> {
    let caller = caller().to_text();

    Ok(ADDRESS_BOOK.with(|book| {
        let book = book.borrow();
        book.get(&caller)
            .map(|user| user.user_addresses.iter().cloned().collect())
            .unwrap_or_default()
    }))
}

#[update]
pub async fn delete_all() -> Result<(), AddressBookError> {
    let caller = caller().to_text();

    ADDRESS_BOOK.with(|book| {
        let mut book = book.borrow_mut();

        if let Some(user) = book.get_mut(&caller) {
            user.user_addresses.clear();
            Ok(())
        } else {
            Ok(())
        }
    })
}

#[query]
pub fn get_config() -> Conf {
    CONFIG.with(|c| c.borrow().clone())
}

#[update]
pub async fn set_config(conf: Conf) -> Result<(), AddressBookError> {
    let caller = caller();

    let current_controllers = get_controllers().await;
    if !current_controllers.contains(&caller) {
        return Err(AddressBookError::Unauthorized);
    }

    CONFIG.with(|c| c.replace(conf));
    Ok(())
}

fn get_user_addresses(book: &HashMap<String, User>, caller: &str) -> Vec<UserAddress> {
    book.get(caller)
        .map(|user| user.user_addresses.iter().cloned().collect())
        .unwrap_or_default()
}

fn validate_no_duplicate_names(
    user_addresses: &HashSet<UserAddress>,
    new_user_address: &UserAddress,
) -> Result<(), AddressBookError> {
    let has_duplicate = user_addresses
        .iter()
        .filter(|address| address.id != new_user_address.id)
        .any(|address| address.name == new_user_address.name);

    if has_duplicate {
        return Err(AddressBookError::DuplicateName);
    }

    Ok(())
}

fn validate_no_duplicate_addresses(
    user_addresses: &HashSet<UserAddress>,
    new_user_address: &UserAddress,
) -> Result<(), AddressBookError> {
    let addresses = new_user_address.addresses.iter()
        .try_fold(HashSet::new(), |mut acc, address| {
            if acc.insert(address) {
                Ok(acc)
            } else {
                Err(AddressBookError::DuplicateAddress)
            }
        })?;

    let has_duplicate = user_addresses
        .iter()
        .filter(|user_address| user_address.id != new_user_address.id)
        .flat_map(|user_address| &user_address.addresses)
        .any(|existing_address| addresses.contains(existing_address));

    if has_duplicate {
        return Err(AddressBookError::DuplicateAddress);
    }

    Ok(())
}

pub async fn get_controllers() -> Vec<Principal> {
    let res: CallResult<(CanisterStatusResponse,)> = call(
        Principal::management_canister(),
        "canister_status",
        (CanisterIdRequest { canister_id: id() },),
    )
    .await;

    return res
        .expect("Inter-canister call to management canister for canister_status returned an empty result.")
        .0.settings.controllers;
}

