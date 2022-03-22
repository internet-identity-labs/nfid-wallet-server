use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::storage;

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

pub type Logs = Vec<Log>;

pub struct LogRepo {}

impl LogRepo {
    pub fn get_all() -> Vec<Log> {
        storage::get::<Logs>().to_vec()
    }

    pub fn get(n: usize) -> Vec<Log> {
        let mut log = storage::get::<Logs>().to_vec();
        log.reverse();
        log.into_iter().take(n).collect()
    }

    pub fn save(log_entry: Log) {
        let logs = storage::get_mut::<Logs>();
        if logs.len() > 500 {
            logs.remove(0);
        }
        logs.push(log_entry)
    }
}