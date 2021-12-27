use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use blake3::Hash;

use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk_macros::*;

use repository::repo::{Account, Device, Persona};
use repository::repo;
use service::{device_service, persona_service};

use crate::http::requests;
use crate::http::response_mapper;
use crate::requests::{HTTPAccountRequest, HTTPVerifyPhoneNumberRequest};
use crate::requests::HTTPAccountUpdateRequest;
use crate::requests::HTTPPersonaUpdateRequest;
use crate::response_mapper::{HttpResponse};
use crate::service::account_service;

mod service;
mod http;
mod repository;

type Topic = String;
type Message = String;

#[derive(Clone, Debug, CandidType, Deserialize)]
struct MessageHttpResponse {
    status_code: u16,
    body: Option<Vec<Message>>,
}


thread_local! {
    static MESSAGE_STORAGE: RefCell<HashMap<Topic, Vec<Message>>> = RefCell::new(HashMap::default());
    static TOKEN_STORAGE: RefCell<HashMap<Hash, Hash>> = RefCell::new(HashMap::default());
}

#[update]
async fn create_account(account_request: HTTPAccountRequest) -> HttpResponse<Account> {
    account_service::create_account(account_request)
}

#[update]
async fn update_account(account_request: HTTPAccountUpdateRequest) -> HttpResponse<Account> {
    account_service::update_account(account_request)
}

#[query]
async fn get_account() -> HttpResponse<Account> {
    account_service::get_account()
}

#[update]
async fn read_devices() -> HttpResponse<Vec<Device>> {
    device_service::read_devices()
}


#[update]
async fn create_device(device: Device) -> HttpResponse<bool> {
    device_service::create_device(device)
}

#[update]
async fn create_persona(persona: Persona) -> HttpResponse<Account> {
    persona_service::create_persona(persona)
}

#[update]
async fn update_persona(request: HTTPPersonaUpdateRequest) -> HttpResponse<Account> { //TODO needs to be refactored
    persona_service::update_persona(request)
}

#[update]
async fn read_personas() -> HttpResponse<Vec<Persona>> {
    persona_service::read_personas()
}

#[pre_upgrade]
fn pre_upgrade() {
    repo::pre_upgrade()
}

#[post_upgrade]
fn post_upgrade() {
    repo::post_upgrade()
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

#[update]
async fn post_token(request: HTTPVerifyPhoneNumberRequest) -> HttpResponse<bool> {
    let phone_number_hash = blake3::hash(request.phone_number.as_bytes());
    let token_hash = blake3::hash(request.token.as_bytes());

    TOKEN_STORAGE.with(|storage| {
        storage.borrow_mut().insert(phone_number_hash, token_hash);
        HttpResponse { status_code: 200, data: Some(true), error: None }
    })
}


fn main() {}
