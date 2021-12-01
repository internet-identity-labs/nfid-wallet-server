use std::cell::{RefCell};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk_macros::{update, query};

type Topic = String;
type Message = String;
type PhoneNumber = String;
type Token = String;
type Error = String;

#[derive(Clone, Debug, CandidType, Deserialize)]
struct MessageHttpResponse {
    status_code: u16,
    body: Option<Vec<Message>>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct HttpResponse<T> {
    data: Option<T>,
    error: Option<Error>,
    status_code: u16,
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

#[query]
async fn verify_phone_number(phone_number: PhoneNumber) -> HttpResponse<bool> {
    HttpResponse { status_code: 200, data: Some(true), error: None }
}

#[query]
async fn verify_token(token: Token) -> HttpResponse<bool> {
    let message: String = String::from("Incorrect token.");
    HttpResponse { status_code: 400, data: None, error: Some(message) }
}

fn main() {
}
