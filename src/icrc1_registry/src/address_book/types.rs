use std::collections::HashSet;
use std::hash::Hash;

use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(CandidType, Deserialize, Clone, Serialize, Debug, Hash, PartialEq)]
pub struct AddressBookConf {
    pub max_user_addresses: u32,
    pub max_name_length: u32,
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug, PartialEq, Eq)]
pub enum AddressBookAddressType {
    IcpAddress,
    IcpPrincipal,
    BTC,
    ETH,
}

impl Hash for AddressBookAddressType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            AddressBookAddressType::IcpAddress => 0.hash(state),
            AddressBookAddressType::IcpPrincipal => 1.hash(state),
            AddressBookAddressType::BTC => 2.hash(state),
            AddressBookAddressType::ETH => 3.hash(state),
        }
    }
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug, Eq)]
pub struct AddressBookAddress {
    pub address_type: AddressBookAddressType,
    pub value: String,
}

impl Hash for AddressBookAddress {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.address_type.hash(state);
        self.value.hash(state);
    }
}

impl PartialEq for AddressBookAddress {
    fn eq(&self, other: &Self) -> bool {
        self.address_type == other.address_type && self.value == other.value
    }
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug, Eq)]
pub struct AddressBookUserAddress {
    pub id: String,
    pub name: String,
    pub addresses: Vec<AddressBookAddress>,
}

impl Hash for AddressBookUserAddress {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for AddressBookUserAddress {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug, PartialEq, Eq)]
pub struct AddressBookUser {
    pub user_addresses: HashSet<AddressBookUserAddress>,
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug, PartialEq, Eq)]
pub enum AddressBookError {
    NameTooLong,
    MaxAddressesReached,
    AddressNotFound,
    DuplicateAddress,
    DuplicateName,
}
