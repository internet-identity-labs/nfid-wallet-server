
#[derive(Clone, Debug, CandidType, Deserialize, Default, PartialEq, Eq, Copy, Hash, Serialize)]
pub struct BaseFields {
    created_date: u64,
    modified_date: u64,
}