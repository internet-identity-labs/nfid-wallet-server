use crate::application_service::is_over_the_limit;
use crate::http::requests::{AccountResponse, PersonaVariant};
use crate::mapper::account_mapper::account_to_account_response;
use crate::mapper::persona_mapper::{persona_request_to_persona, persona_to_persona_response};
use crate::repo::is_anchor_exists;
use crate::repository::repo::{PersonaRepo};
use crate::response_mapper::{HttpResponse, to_error_response, to_success_response};
use crate::util::validation_util::validate_frontend_length;

pub fn create_persona(persona_r: PersonaVariant) -> HttpResponse<AccountResponse> {
    if !validate_frontend_length(&persona_r) {
        return to_error_response("Invalid persona");
    }
    let created_persona = persona_request_to_persona(persona_r);
    if created_persona.anchor.is_some() && is_anchor_exists(created_persona.anchor.unwrap()) {
        return to_error_response("It's impossible to link this II anchor, please try another one.");
    }

    if is_over_the_limit(&created_persona.domain) {
        return to_error_response("It's impossible to link this domain. Over limit.");
    }

    match PersonaRepo::store_persona(created_persona) {
        Some(t) => {
            to_success_response(account_to_account_response(t))
        }
        None => to_error_response("Unable to store persona.")
    }
}

pub fn read_personas() -> HttpResponse<Vec<PersonaVariant>> {
    match PersonaRepo::get_personas() {
        Some(personas) => {
            let personas_r = personas.iter()
                .map(|l| persona_to_persona_response(l.to_owned()))
                .collect();
            to_success_response(personas_r)
        }
        None => to_error_response("Unable to find Account.")
    }
}
