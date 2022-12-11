use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, CandidType, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum State {
    APPROVED,
    REJECTED,
    PENDING,
    CANCELED
}
