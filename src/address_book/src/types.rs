use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

#[derive(CandidType, Debug, Clone, Deserialize)]
pub struct CanisterIdRequest {
    #[serde(rename = "canister_id")]
    pub canister_id: Principal,
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug, Hash, PartialEq)]
pub struct Conf {
    pub max_user_addresses: u32,
    pub max_name_length: u32,
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug, PartialEq, Eq)]
pub enum AddressType {
    IcpAddress,
    IcpPrincipal,
    BTC,
    ETH,
}

impl Hash for AddressType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            AddressType::IcpAddress => 0.hash(state),
            AddressType::IcpPrincipal => 1.hash(state),
            AddressType::BTC => 2.hash(state),
            AddressType::ETH => 3.hash(state),
        }
    }
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug, Eq)]
pub struct Address {
    pub address_type: AddressType,
    pub value: String,
}

impl Hash for Address {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.address_type.hash(state);
        self.value.hash(state);
    }
}

impl PartialEq for Address {
    fn eq(&self, other: &Self) -> bool {
        self.address_type == other.address_type && self.value == other.value
    }
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug, Eq)]
pub struct UserAddress {
    pub id: String,
    pub name: String,
    pub addresses: Vec<Address>,
}

impl Hash for UserAddress {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for UserAddress {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug, PartialEq, Eq)]
pub struct User {
    pub user_addresses: HashSet<UserAddress>,
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug, PartialEq, Eq)]
pub enum AddressBookError {
    NameTooLong,
    MaxAddressesReached,
    AddressNotFound,
    DuplicateAddress,
    DuplicateName,
    Unauthorized,
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct Memory {
    pub data: HashMap<String, User>,
    pub config: Conf,
}
