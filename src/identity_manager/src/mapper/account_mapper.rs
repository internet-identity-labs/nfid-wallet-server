use crate::mapper::persona_mapper::persona_to_persona_response;
use crate::repository::repo::Account;
use crate::http::requests::AccountResponse;
use crate::{AccessPoint, HTTPAccountRequest};
use crate::repo::Persona;

pub fn account_to_account_response(account: Account) -> AccountResponse {
    let personas_r = account.personas.iter()
        .map(|l| persona_to_persona_response(l.clone()))
        .collect();
    AccountResponse {
        anchor: account.anchor,
        principal_id: account.principal_id,
        name: account.name,
        phone_number: account.phone_number,
        access_points: account.access_points,
        personas: personas_r,
    }
}

pub fn account_request_to_account(account_request: HTTPAccountRequest) -> Account {
    let principal_id = ic_cdk::api::caller().to_text();
    let access_points: Vec<AccessPoint> = Vec::new();
    let personas: Vec<Persona> = Vec::new();
    Account {
        anchor: account_request.anchor,
        principal_id,
        name: account_request.name,
        phone_number: account_request.phone_number,
        access_points: access_points,
        personas,
    }
}
