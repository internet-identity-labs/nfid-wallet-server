use crate::mapper::persona_mapper::persona_to_persona_response;
use crate::http::requests::AccountResponse;
use crate::{AccountRequest};
use crate::repository::account_repo::Account;
use crate::repository::persona_repo::Persona;
use crate::repository::repo::BasicEntity;
use crate::service::ic_service;

pub fn account_to_account_response(account: Account) -> AccountResponse {
    let personas_r = account.personas.iter()
        .map(|l| persona_to_persona_response(l.clone()))
        .collect();
    AccountResponse {
        anchor: account.anchor,
        principal_id: account.principal_id,
        name: account.name,
        phone_number: account.phone_number,
        personas: personas_r,
        access_points: account.access_points.into_iter().collect()
    }
}

pub fn account_request_to_account(account_request: AccountRequest) -> Account {
    let principal_id = ic_service::get_caller().to_text();
    let personas: Vec<Persona> = Vec::new();
    Account {
        anchor: account_request.anchor,
        principal_id,
        name: None,
        phone_number: None,
        phone_number_sha2: None,
        personas,
        access_points: Default::default(),
        base_fields: BasicEntity::new(),
    }
}
