use std::cell::{RefCell};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk_macros::{update};


type Topic = String;
type Message = String;


#[derive(Clone, Debug, CandidType, Deserialize)]
struct MessageHttpResponse {
    status_code: u16,
    body: Option<Vec<Message>>,
}

thread_local! {
    static MESSAGE_STORAGE: RefCell<HashMap<Topic, Vec<Message>>> = RefCell::new(HashMap::default());
}


#[update]
async fn post_messages(topic: Topic, mut messages: Vec<Message>) -> MessageHttpResponse {
    MESSAGE_STORAGE.with(|storage| {
        return match storage.borrow_mut().entry(topic) {
            Entry::Occupied(mut o) => {
                o.get_mut().append(&mut messages);
                MessageHttpResponse { status_code: 200, body: None }
            },
            Entry::Vacant(_v) => {
                MessageHttpResponse { status_code: 404, body: None }
            }
        }
    })
}

#[update]
async fn get_messages(topic: Topic) -> MessageHttpResponse {
    MESSAGE_STORAGE.with(|storage| {
        return match storage.borrow_mut().entry(topic) {
            Entry::Occupied(mut o) => {
                let messages = o.get().to_vec();
                o.get_mut().clear();
                MessageHttpResponse { status_code: 200, body: Some(messages) }
            },
            Entry::Vacant(_v) => {
                MessageHttpResponse { status_code: 404, body: None }
            }
        }
    })
}

#[update]
async fn create_topic(topic: Topic) -> MessageHttpResponse {
    MESSAGE_STORAGE.with(|storage| {
        return match storage.borrow_mut().entry(topic) {
            Entry::Occupied(_o) => {
                MessageHttpResponse { status_code: 409, body: None }
            },
            Entry::Vacant(v) => {
                v.insert(Vec::new());
                MessageHttpResponse { status_code: 200, body: None }
            }
        }
    })
}

#[update]
async fn delete_topic(topic: Topic) -> MessageHttpResponse {
    MESSAGE_STORAGE.with(|storage| {
        return match storage.borrow_mut().entry(topic) {
            Entry::Occupied(o) => {
                o.remove_entry();
                MessageHttpResponse { status_code: 200, body: None }
            },
            Entry::Vacant(_v) => {
                MessageHttpResponse { status_code: 404, body: None }
            }
        }
    })
}

fn main() {
}
