use ic_cdk::{caller};
use ic_ledger_types::{AccountIdentifier, Subaccount};

pub fn caller_to_address() -> String {
    return AccountIdentifier::new(&caller(), &Subaccount([1; 32])).to_string();
}