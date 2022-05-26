use std::collections::HashSet;
use crate::{DeviceData, KeyType, Principal};
use crate::AccessPoint;
use crate::mapper::persona_mapper::persona_to_persona_response;
use crate::http::requests::AccountResponse;
use crate::{AccountRequest};
use crate::mapper::access_point_mapper::access_point_to_access_point_response;
use crate::repository::account_repo::Account;
use crate::repository::persona_repo::Persona;
use crate::repository::repo::BasicEntity;

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
        access_points: account.access_points.into_iter()
            .map(access_point_to_access_point_response)
            .collect(),
    }
}

pub fn account_request_to_account(principal_id: String, account_request: AccountRequest, devices: Vec<DeviceData>) -> Account {
    let personas: Vec<Persona> = Vec::new();
    let basic = BasicEntity::new();
    let pub_key = get_recovery_phrase(devices);
    let access_points: HashSet<AccessPoint> = get_access_points(pub_key, basic);

    Account {
        anchor: account_request.anchor,
        principal_id,
        name: None,
        phone_number: None,
        phone_number_sha2: None,
        personas,
        access_points,
        base_fields: basic,
    }
}

fn get_recovery_phrase(devices: Vec<DeviceData>) -> Option<String> {
    for device in devices {
        match device.key_type {
            KeyType::SeedPhrase => {
                return Some(Principal::self_authenticating(device.pubkey).to_text())
            },
            _ => (),
        }
    }
    None
}

fn get_access_points(recovery_phrase: Option<String>, basic: BasicEntity) -> HashSet<AccessPoint> {
    recovery_phrase
        .map(|principal_text| {
             AccessPoint {
                principal_id: principal_text,
                icon: Some("recovery".to_string()),
                device: Some("recovery".to_string()),
                browser: Some("".to_string()),
                last_used: Some(basic.get_created_date().clone()),
                base_fields: basic
            }
        })
        .map(|access_point| {
            let mut set = HashSet::new();
            set.insert(access_point);
            set
        })
        .unwrap_or(Default::default())
}
