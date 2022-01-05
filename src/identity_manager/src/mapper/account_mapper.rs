use crate::mapper::persona_mapper::persona_to_persona_response;
use crate::repository::repo::Account;
use crate::http::requests::AccountResponse;

pub fn account_to_account_response(account: Account) -> AccountResponse {
    let personas_r = account.personas.iter()
        .map(|l| persona_to_persona_response(l.clone()))
        .collect();
    AccountResponse {
        principal_id: account.principal_id,
        name: account.name,
        phone_number: account.phone_number,
        email: account.email,
        devices: account.devices,
        personas: personas_r,
        is_seed_phrase_copied: account.is_seed_phrase_copied,
    }
}
