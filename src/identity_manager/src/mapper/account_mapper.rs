use crate::mapper::persona_mapper::persona_to_persona_response;
use crate::repository::repo::Account;
use crate::http::requests::AccountRR;

pub fn account_to_account_response(account: Account) -> AccountRR {
    let personas_r = account.personas.iter()
        .map(|l| persona_to_persona_response(l.clone()))
        .collect();
    AccountRR {
        principal_id: account.principal_id,
        name: account.name,
        phone_number: account.phone_number,
        email: account.email,
        devices: account.devices,
        personas: personas_r,
    }
}
