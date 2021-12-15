use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk_macros::*;

use repository::repo::{Account, Device, Persona};
use repository::repo;
use service::{device_service, persona_service};

use crate::http::requests;
use crate::http::response_mapper;
use crate::requests::HTTPAccountRequest;
use crate::requests::HTTPAccountUpdateRequest;
use crate::requests::HTTPPersonaUpdateRequest;
use crate::response_mapper::{HttpResponse, to_error_response};
use crate::service::account_service;

mod service;
mod http;
mod repository;

type Topic = String;
type Message = String;
type PhoneNumber = String;
type Token = String;

#[derive(Clone, Debug, CandidType, Deserialize)]
struct MessageHttpResponse {
    status_code: u16,
    body: Option<Vec<Message>>,
}


thread_local! {
    static MESSAGE_STORAGE: RefCell<HashMap<Topic, Vec<Message>>> = RefCell::new(HashMap::default());
    static TOKEN_STORAGE: RefCell<HashMap<PhoneNumber, Token>> = RefCell::new(HashMap::default());
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

#[query]
async fn verify_phone_number(phone_number: PhoneNumber) -> HttpResponse<bool> {
    HttpResponse { status_code: 200, data: Some(true), error: None }
}

#[query]
async fn verify_token(phone_number: PhoneNumber, token: Token) -> HttpResponse<bool> {
    TOKEN_STORAGE.with(|storage| {
        return match storage.borrow_mut().entry(phone_number) {
            Entry::Occupied(mut o) => {
                return if token.eq(o.get_mut()) {
                    HttpResponse { status_code: 200, data: Some(true), error: None }
                } else {
                    to_error_response("Token has been expired.")
                };
            }
            Entry::Vacant(_v) => {
                to_error_response("Not found.")
            }
        };
    })
}

#[update]
async fn post_token(phone_number: PhoneNumber, token: Token) -> HttpResponse<bool> {
    TOKEN_STORAGE.with(|storage| {
        storage.borrow_mut().insert(phone_number, token);
        HttpResponse { status_code: 200, data: Some(true), error: None }
    })
}


fn main() {}
