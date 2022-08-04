use std::cell::RefCell;

use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk_macros::*;
use canister_api_macros::{collect_metrics};


type Topic = String;
type Message = String;

use structure::ttlhashmap::{TtlHashMap};
use std::time::{Duration};

mod structure;

#[derive(Clone, Debug, CandidType, Deserialize)]
struct MessageHttpResponse {
    status_code: u16,
    body: Option<Vec<Message>>,
    error: Option<String>,
}

const DEFAULT_TOKEN_TTL: Duration = Duration::from_secs(90);

thread_local! {
    static MESSAGE_STORAGE: RefCell<TtlHashMap<Topic, Vec<Message>>> = RefCell::new(TtlHashMap::new(DEFAULT_TOKEN_TTL));
    static HEARTBEAT_STORAGE: RefCell<u64> = RefCell::new(1);
}

#[query]
async fn ping() -> () {}

#[update]
#[collect_metrics]
async fn post_messages(topic: Topic, mut messages: Vec<Message>) -> MessageHttpResponse {
    MESSAGE_STORAGE.with(|storage| {
        let mut st = storage.borrow_mut();
        match st.get(&topic) {
            Some(o) => {
                let len = o.len() + messages.clone().len();
                if len > 30 {
                    return MessageHttpResponse {
                        status_code: 400,
                        body: None,
                        error: Some(String::from("More than 30 messages in channel")),
                    };
                }
                let k: Vec<Message> = messages
                    .clone()
                    .into_iter()
                    .filter(|l| l.as_str().len() > 10000)
                    .collect();
                if k.len() > 0 {
                    return MessageHttpResponse {
                        status_code: 400,
                        body: None,
                        error: Some(String::from("One of messages is more than 10000 chars")),
                    };
                }
                messages.append(&mut o.clone());
            }
            None => {
                return MessageHttpResponse {
                    status_code: 404,
                    body: None,
                    error: Some(String::from("No such topic")),
                };
            }
        };
        st.insert(topic, messages.clone());
        return MessageHttpResponse {
            status_code: 200,
            body: Some(messages),
            error: None,
        };
    })
}

#[query]
#[collect_metrics]
async fn get_messages(topic: Topic) -> MessageHttpResponse {
    let mut rsp = MessageHttpResponse {
        status_code: 0,
        body: None,
        error: None,
    };
    MESSAGE_STORAGE.with(|storage| {
        let mut st = storage.borrow_mut();
        match st.get(&topic) {
            Some(messages) => {
                rsp = MessageHttpResponse {
                    status_code: 200,
                    body: Some(messages.clone()),
                    error: None,
                }
            }
            None => {
                rsp = MessageHttpResponse {
                    status_code: 404,
                    body: None,
                    error: Some(String::from("No such topic")),
                }
            }
        };
        st.insert(topic, Vec::new());
    });
    rsp
}

#[update]
#[collect_metrics]
async fn create_topic(topic: Topic) -> MessageHttpResponse {
    MESSAGE_STORAGE.with(|storage| {
        let mut st = storage.borrow_mut();
        return match st.get(&topic) {
            Some(_o) => {
                MessageHttpResponse {
                    status_code: 409,
                    body: None,
                    error: Some(String::from("Topic exist")),
                }
            }
            None => {
                st.insert(topic, Vec::new());
                MessageHttpResponse {
                    status_code: 200,
                    body: None,
                    error: None,
                }
            }
        };
    })
}

#[update]
async fn delete_topic(topic: Topic) -> MessageHttpResponse {
    MESSAGE_STORAGE.with(|storage| {
        return match storage.borrow_mut().remove(&topic) {
            Some(_o) => {
                MessageHttpResponse {
                    status_code: 200,
                    body: None,
                    error: None,
                }
            }
            None => {
                MessageHttpResponse {
                    status_code: 404,
                    body: None,
                    error: Some(String::from("No such topic")),
                }
            }
        };
    })
}

#[heartbeat]
async fn clean() {
    HEARTBEAT_STORAGE.with(|hb| {
        let mut heart_tick = hb.borrow_mut();
        if (*heart_tick % DEFAULT_TOKEN_TTL.as_secs()) == 0 {
            MESSAGE_STORAGE.with(|storage| {
                storage.borrow_mut().cleanup();
            });
            *heart_tick = 1;
        } else {
            *heart_tick += 1;
        }
    });
}

#[pre_upgrade]
fn pre_upgrade() {}

#[post_upgrade]
fn post_upgrade() {}


#[ic_cdk_macros::query(name = "getCanisterMetrics")]
pub async fn get_canister_metrics(parameters: canistergeek_ic_rust::api_type::GetMetricsParameters) -> Option<canistergeek_ic_rust::api_type::CanisterMetrics<'static>> {
    canistergeek_ic_rust::monitor::get_metrics(&parameters)
}

#[ic_cdk_macros::update(name = "collectCanisterMetrics")]
pub async fn collect_canister_metrics() -> () {
    canistergeek_ic_rust::monitor::collect_metrics();
}

#[ic_cdk_macros::query(name = "getCanisterLog")]
pub async fn get_canister_log(request: Option<canistergeek_ic_rust::api_type::CanisterLogRequest>) -> Option<canistergeek_ic_rust::api_type::CanisterLogResponse<'static>> {
    canistergeek_ic_rust::logger::get_canister_log(request)
}

fn main() {}
