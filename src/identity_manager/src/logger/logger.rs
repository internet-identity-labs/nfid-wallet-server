use candid::{CandidType, Deserialize};

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Log {
    pub level: LogLevel,
    pub log: String,
    pub timestamp: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum LogLevel {
    #[serde(rename = "ERROR")]
    ERROR,
    #[serde(rename = "INFO")]
    INFO,
}

//deprecated
pub type Logs = Vec<Log>;
