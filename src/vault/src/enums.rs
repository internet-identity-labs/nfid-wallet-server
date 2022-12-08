use serde::{Deserialize, Serialize};
use candid::{candid_method, CandidType, Principal};


#[derive(Clone, Debug, CandidType, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum State {
    APPROVED,
    REJECTED,
    PENDING
}
