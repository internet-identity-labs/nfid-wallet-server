use ic_cdk::storage;
use ic_cdk::export::candid::{CandidType, Deserialize};

#[derive(Debug, Deserialize, CandidType, Clone)]
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
    INFO
}

pub type Logs = Vec<Log>;

pub struct LogRepo {}

impl LogRepo {
    pub fn get_all() -> Vec<Log> {
        storage::get::<Logs>().to_vec()
    }

    pub fn get(n: usize) -> Vec<Log> {
        storage::get::<Logs>().to_vec()
            .into_iter().take(n).collect()
    }

    pub fn save(log_entry: Log) {
        let mut logs = storage::get_mut::<Logs>();
        logs.push(log_entry)
    }
}