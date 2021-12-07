use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::collections::hash_map::Entry;

use ic_cdk::{storage};
use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk_macros::*;

type Topic = String;
type Message = String;
type PhoneNumber = String;
type Token = String;
type Error = String;
type Accounts = BTreeMap<String, Account>;

#[derive(Clone, Debug, CandidType, Deserialize)]
struct MessageHttpResponse {
    status_code: u16,
    body: Option<Vec<Message>>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct Account {
    principal_id: String,
    name: String,
    phone_number: String,
    email: String,
    devices: Vec<Device>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct HttpResponse<T> {
    data: Option<T>,
    error: Option<Error>,
    status_code: u16,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct HTTPAccountRequest {
    name: String,
    phone_number: String,
    email: String,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct HTTPAccountUpdateRequest {
    name: Option<String>,
    phone_number: Option<String>,
    email: Option<String>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct Device {
    pub_key_hash: String,
    last_used: String,
    make: String,
    model: String,
    browser: String,
}

thread_local! {
    static MESSAGE_STORAGE: RefCell<HashMap<Topic, Vec<Message>>> = RefCell::new(HashMap::default());
}

#[update]
async fn create_account(account_request: HTTPAccountRequest) -> HttpResponse<Account> {
    let princ = &ic_cdk::api::caller().to_text();
    let devices: Vec<Device> = Vec::new();
    let acc = Account {
        principal_id: princ.clone(),
        name: account_request.name,
        phone_number: account_request.phone_number,
        email: account_request.email,
        devices,
    };
    let accounts = storage::get_mut::<Accounts>();
    accounts.insert(princ.clone(), acc.clone());
    HttpResponse {
        data: Option::from(acc),
        error: None,
        status_code: 200,
    }
}

#[update]
async fn update_account(account_request: HTTPAccountUpdateRequest) -> HttpResponse<Account> {
    let princ = &ic_cdk::api::caller().to_text();
    let accounts = storage::get_mut::<Accounts>();
    match accounts.get(princ) {
        Some(acc) => {
            let mut new_acc = acc.clone();
            if !account_request.email.is_none() {
                new_acc.email = account_request.email.unwrap();
            }
            if !account_request.phone_number.is_none() {
                new_acc.phone_number = account_request.phone_number.unwrap();
            }
            if !account_request.name.is_none() {
                new_acc.name = account_request.name.unwrap();
            }
            accounts.insert(princ.clone(), new_acc.clone());
            HttpResponse {
                data: Option::from(new_acc),
                error: None,
                status_code: 200,
            }
        }
        None => to_error_response("Unable to find Account.")
    }
}

#[query]
async fn get_account() -> HttpResponse<Account> {
    let princ = &ic_cdk::api::caller().to_text();
    let accounts = storage::get_mut::<Accounts>();
    match accounts.get(princ) {
        Some(content) => HttpResponse {
            data: Some(content.clone()),
            error: None,
            status_code: 200,
        },
        None => to_error_response("Unable to find Account.")
    }
}

#[update]
async fn read_devices() -> HttpResponse<Vec<Device>> {
    let princ = &ic_cdk::api::caller().to_text();
    let accounts = storage::get_mut::<Accounts>();
    match accounts.get(princ) {
        Some(content) => HttpResponse {
            data: Some(content.clone().devices),
            error: None,
            status_code: 200,
        },
        None => to_error_response("Unable to find Account.")
    }
}


#[update]
async fn create_device(device: Device) -> HttpResponse<bool> {
    let princ = &ic_cdk::api::caller().to_text();
    let accounts = storage::get_mut::<Accounts>();
    match accounts.get(princ) {
        Some(acc) => {
            let mut new_acc = acc.clone();
            let mut devices = new_acc.devices;
            devices.push(device);
            new_acc.devices = devices;
            accounts.insert(princ.clone(), new_acc);
            HttpResponse {
                data: Some(true),
                error: None,
                status_code: 200,
            }
        }
        None => to_error_response("Unable to find Account.")
    }
}

#[pre_upgrade]
fn pre_upgrade() {
    let mut vec = Vec::new();
    for p in storage::get_mut::<Accounts>().iter() {
        vec.push(p.1.clone());
    }
    storage::stable_save((vec, )).unwrap();
}

#[post_upgrade]
fn post_upgrade() {
    let (old_users, ): (Vec<Account>, ) = storage::stable_restore().unwrap();
    for u in old_users {
        storage::get_mut::<Accounts>().insert(u.clone().principal_id, u.clone());
    }
}


#[update]
async fn post_messages(topic: Topic, mut messages: Vec<Message>) -> MessageHttpResponse {
    MESSAGE_STORAGE.with(|storage| {
        return match storage.borrow_mut().entry(topic) {
            Entry::Occupied(mut o) => {
                o.get_mut().append(&mut messages);
                MessageHttpResponse { status_code: 200, body: None }
            }
            Entry::Vacant(_v) => {
                MessageHttpResponse { status_code: 404, body: None }
            }
        };
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
            }
            Entry::Vacant(_v) => {
                MessageHttpResponse { status_code: 404, body: None }
            }
        };
    })
}

#[update]
async fn create_topic(topic: Topic) -> MessageHttpResponse {
    MESSAGE_STORAGE.with(|storage| {
        return match storage.borrow_mut().entry(topic) {
            Entry::Occupied(_o) => {
                MessageHttpResponse { status_code: 409, body: None }
            }
            Entry::Vacant(v) => {
                v.insert(Vec::new());
                MessageHttpResponse { status_code: 200, body: None }
            }
        };
    })
}

#[update]
async fn delete_topic(topic: Topic) -> MessageHttpResponse {
    MESSAGE_STORAGE.with(|storage| {
        return match storage.borrow_mut().entry(topic) {
            Entry::Occupied(o) => {
                o.remove_entry();
                MessageHttpResponse { status_code: 200, body: None }
            }
            Entry::Vacant(_v) => {
                MessageHttpResponse { status_code: 404, body: None }
            }
        };
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

fn to_error_response<T>(x: &str) -> HttpResponse<T> {
    HttpResponse {
        data: None,
        error: Some(String::from(x)),
        status_code: 404,
    }
}

fn main() {}
