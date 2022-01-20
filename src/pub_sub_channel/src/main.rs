use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk_macros::*;

type Topic = String;
type Message = String;

use structure::ttlhashmap::{AutoClean, TtlHashMap};
use std::time::{Duration};
use ic_cdk::print;

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
}

#[update]
async fn post_messages(topic: Topic, mut messages: Vec<Message>) -> MessageHttpResponse {
    let princ = &ic_cdk::api::caller().to_text();
    if (princ.len() < 10) {
        return MessageHttpResponse {
            status_code: 401,
            body: None,
            error: Some(String::from("User is anonymous")),
        };
    }
    MESSAGE_STORAGE.with(|storage| {
        let mut st = storage.borrow_mut();
        match st.get(&topic) {
            Some(mut o) => {
                let len = o.len() + messages.clone().len();
                if len > 10 {
                    return MessageHttpResponse {
                        status_code: 400,
                        body: None,
                        error: Some(String::from("More than 10 messages in channel")),
                    };
                }
                let k: Vec<Message> = messages
                    .clone()
                    .into_iter()
                    .filter(|l| l.as_str().len() > 1000)
                    .collect();
                if (k.len() > 0) {
                    return MessageHttpResponse {
                        status_code: 400,
                        body: None,
                        error: Some(String::from("One of messages is more than 1000 chars")),
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

#[update]
async fn get_messages(topic: Topic) -> MessageHttpResponse {
    MESSAGE_STORAGE.with(|storage| {
        return match storage.borrow_mut().get(&topic) {
            Some(mut messages) => {
                MessageHttpResponse {
                    status_code: 200,
                    body: Some(messages.clone()),
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

#[update]
async fn create_topic(topic: Topic) -> MessageHttpResponse {
    let princ = &ic_cdk::api::caller().to_text();
    if (princ.len() < 10) {
        return MessageHttpResponse {
            status_code: 401,
            body: None,
            error: Some(String::from("User is anonymous")),
        };
    }
    MESSAGE_STORAGE.with(|storage| {
        let mut st = storage.borrow_mut();
        return match st.get(&topic) {
            Some(o) => {
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
    let princ = &ic_cdk::api::caller().to_text();
    if (princ.len() < 10) {
        return MessageHttpResponse {
            status_code: 401,
            body: None,
            error: Some(String::from("User is anonymous")),
        };
    }
    MESSAGE_STORAGE.with(|storage| {
        return match storage.borrow_mut().remove(&topic) {
            Some(o) => {
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

#[pre_upgrade]
fn pre_upgrade() {}

#[post_upgrade]
fn post_upgrade() {}

fn main() {}