use crate::http::requests::{AccountResponse, PersonaVariant};
use crate::mapper::account_mapper::account_to_account_response;
use crate::mapper::persona_mapper::{persona_request_to_persona, persona_to_persona_response};
use crate::repository::repo::{Persona, PersonaRepo};
use crate::response_mapper::{HttpResponse, to_error_response, to_success_response};
use crate::util::validation_util::validate_frontend_length;

pub fn create_persona(persona_r: PersonaVariant) -> HttpResponse<AccountResponse> {
    if !validate_frontend_length(persona_r.clone()) {
        return to_error_response("Invalid persona");
    }
    match PersonaRepo::get_personas()
    {
        Some(mut personas) => {
            let created_persona = persona_request_to_persona(persona_r.clone());
            personas.push(created_persona.clone());
            match PersonaRepo::store_personas(personas) {
                Some(t) => {
                    to_success_response(account_to_account_response(t.clone()))
                }
                None => to_error_response("Unable to store persona.")
            }
        }
        None => to_error_response("Unable to find Account.")
    }
}

pub fn read_personas() -> HttpResponse<Vec<PersonaVariant>> {
    match PersonaRepo::get_personas() {
        Some(personas) => {
            let personas_r = personas.iter()
                .map(|l| persona_to_persona_response(l.clone()))
                .collect();
            to_success_response(personas_r)
        }
        None => to_error_response("Unable to find Account.")
    }
}
