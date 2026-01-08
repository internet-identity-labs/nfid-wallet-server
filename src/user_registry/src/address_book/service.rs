use std::collections::{HashMap, HashSet};
use ic_cdk::caller;

use super::types::{AddressBookError, AddressBookUser, AddressBookUserAddress};
use crate::{ADDRESS_BOOK, ADDRESS_BOOK_CONFIG};

pub async fn save(user_address: AddressBookUserAddress) -> Result<Vec<AddressBookUserAddress>, AddressBookError> {
    let caller = caller().to_text();

    let (max_name_length, max_addresses) = ADDRESS_BOOK_CONFIG.with(|c| {
        let conf = c.borrow();
        (conf.max_name_length, conf.max_user_addresses)
    });

    if user_address.name.len() > max_name_length as usize {
        return Err(AddressBookError::NameTooLong);
    }

    ADDRESS_BOOK.with(|book| {
        let mut book = book.borrow_mut();
        let user = book.entry(caller.clone()).or_insert_with(|| AddressBookUser {
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

pub async fn delete(id: String) -> Result<Vec<AddressBookUserAddress>, AddressBookError> {
    let caller = caller().to_text();

    ADDRESS_BOOK.with(|book| {
        let mut book = book.borrow_mut();

        if let Some(user) = book.get_mut(&caller) {
            let temp_address = AddressBookUserAddress {
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

pub async fn find_all() -> Result<Vec<AddressBookUserAddress>, AddressBookError> {
    let caller = caller().to_text();

    Ok(ADDRESS_BOOK.with(|book| {
        let book = book.borrow();
        book.get(&caller)
            .map(|user| user.user_addresses.iter().cloned().collect())
            .unwrap_or_default()
    }))
}

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

fn get_user_addresses(book: &HashMap<String, AddressBookUser>, caller: &str) -> Vec<AddressBookUserAddress> {
    book.get(caller)
        .map(|user| user.user_addresses.iter().cloned().collect())
        .unwrap_or_default()
}

fn validate_no_duplicate_names(
    user_addresses: &HashSet<AddressBookUserAddress>,
    new_user_address: &AddressBookUserAddress,
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
    user_addresses: &HashSet<AddressBookUserAddress>,
    new_user_address: &AddressBookUserAddress,
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
