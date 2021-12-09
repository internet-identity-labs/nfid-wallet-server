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
type PrincipalIndex = BTreeMap<String, String>;

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
    personas: Vec<Persona>,
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
struct HTTPPersonaUpdateRequest {
    name: Option<String>,
    is_root: Option<bool>,
    is_seed_phrase_copied: Option<bool>,
    is_ii_anchor: Option<bool>,
    anchor: Option<String>,
    principal_id: String,
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

#[derive(Clone, Debug, CandidType, Deserialize)]
struct Persona {
    name: String,
    is_root: bool,
    is_seed_phrase_copied: bool,
    is_ii_anchor: bool,
    anchor: String,
    principal_id: String,
}

thread_local! {
    static MESSAGE_STORAGE: RefCell<HashMap<Topic, Vec<Message>>> = RefCell::new(HashMap::default());
}

#[update]
async fn create_account(account_request: HTTPAccountRequest) -> HttpResponse<Account> {
    let princ = &ic_cdk::api::caller().to_text();
    let devices: Vec<Device> = Vec::new();
    let personas: Vec<Persona> = Vec::new();
    let acc = Account {
        principal_id: princ.clone(),
        name: account_request.name,
        phone_number: account_request.phone_number,
        email: account_request.email,
        devices,
        personas,
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
    match accounts.get(get_principal(princ)) {
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
    match accounts.get(get_principal(princ)) {
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
    match accounts.get(get_principal(princ)) {
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
    match accounts.get(get_principal(princ)) {
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

#[update]
async fn create_persona(persona: Persona) -> HttpResponse<Account> {
    let princ = &ic_cdk::api::caller().to_text();
    let accounts = storage::get_mut::<Accounts>();
    match accounts.get(get_principal(princ)) {
        Some(acc) => {
            let mut new_acc = acc.clone();
            let mut personas = new_acc.personas;
            let required_id = persona.principal_id.clone();
            for persona in personas.clone().iter() {
                if persona.principal_id == required_id {
                    return to_error_response("Persona already exists")
                }
            }
            personas.push(persona.clone());
            new_acc.personas = personas;
            accounts.insert(princ.clone(), new_acc.clone());
            store_principal(persona.principal_id, princ.clone());
            HttpResponse {
                data: Some(new_acc),
                error: None,
                status_code: 200,
            }
        }
        None => to_error_response("Unable to find Account.")
    }
}

#[update]
async fn update_persona(request: HTTPPersonaUpdateRequest) -> HttpResponse<Account> { //TODO needs to be refactored
    let princ = &ic_cdk::api::caller().to_text();
    let accounts = storage::get_mut::<Accounts>();
    match accounts.get(get_principal(princ)) {
        Some(acc) => {
            let mut new_acc = acc.clone();
            let required_id = request.clone().principal_id.clone();
            let mut updated_persona = Persona {
                name: "".to_string(),
                is_root: false,
                is_seed_phrase_copied: false,
                is_ii_anchor: false,
                anchor: "".to_string(),
                principal_id: "".to_string(),
            };
            let mut updated_personas: Vec<Persona> = Vec::new();
            for persona in new_acc.personas.clone().iter() {
                if persona.principal_id == required_id {
                    updated_persona = persona.clone();
                } else { updated_personas.push(persona.clone()) }
            }
            if updated_persona.principal_id == "" {
                return to_error_response("Unable to find Persona.");
            }
            if request.name.is_some() {
                updated_persona.name = request.name.clone().unwrap();
            }
            if !request.is_root.is_none() {
                updated_persona.is_root = request.is_root.unwrap();
            }
            if !request.is_seed_phrase_copied.is_none() {
                updated_persona.is_seed_phrase_copied = request.is_seed_phrase_copied.unwrap();
            }
            if !request.is_ii_anchor.is_none() {
                updated_persona.is_ii_anchor = request.is_ii_anchor.unwrap();
            }
            if request.anchor.is_some() {
                updated_persona.anchor = request.anchor.clone().unwrap();
            }
            updated_personas.push(updated_persona.clone());
            new_acc.personas = updated_personas;
            accounts.insert(princ.clone(), new_acc.clone());
            HttpResponse {
                data: Some(new_acc),
                error: None,
                status_code: 200,
            }
        }
        None => to_error_response("Unable to find Account.")
    }
}

#[update]
async fn read_personas() -> HttpResponse<Vec<Persona>> {
    let princ = &ic_cdk::api::caller().to_text();
    let accounts = storage::get_mut::<Accounts>();
    match accounts.get(get_principal(princ)) {
        Some(acc) => {
            HttpResponse {
                data: Some(acc.personas.clone()),
                error: None,
                status_code: 200,
            }
        }
        None => to_error_response("Unable to find Account.")
    }
}

fn store_principal(root_id: String, persona_id: String) {
    let principals = storage::get_mut::<PrincipalIndex>();
    principals.insert(root_id.clone(), persona_id.clone());
}

fn get_principal(persona_id: &str) -> &str {
    let principals = storage::get_mut::<PrincipalIndex>();
    match principals.get(persona_id) {
        Some(principal_id) => {
            principal_id
        }
        None => {
            persona_id
        }
    }
}

#[pre_upgrade]
fn pre_upgrade() {
    let mut accounts = Vec::new();
    for p in storage::get_mut::<Accounts>().iter() {
        accounts.push(p.1.clone());
    }
    storage::stable_save((accounts, )).unwrap();
}

#[post_upgrade]
fn post_upgrade() {
    let (old_accs, ): (Vec<Account>, ) = storage::stable_restore().unwrap();
    for u in old_accs {
        storage::get_mut::<Accounts>().insert(u.clone().principal_id, u.clone());
        for persona in u.personas.clone() {
            storage::get_mut::<PrincipalIndex>().insert(persona.principal_id.clone(), u.clone().principal_id);
        }
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
