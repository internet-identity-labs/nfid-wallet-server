use crate::http::requests::{AccountResponse, PersonaRequest, PersonaResponse};
use crate::mapper::account_mapper::account_to_account_response;
use crate::mapper::persona_mapper::{persona_request_to_persona, persona_to_persona_response};
use crate::repository::repo::{Persona, PersonaRepo, PrincipalIndex};
use crate::requests::HTTPPersonaUpdateRequest;
use crate::response_mapper::{HttpResponse, to_error_response, to_success_response};

pub fn create_persona(persona_r: PersonaRequest) -> HttpResponse<AccountResponse> {
    match PersonaRepo::get_personas()
    {
        Some(mut personas) => {
            for persona in personas.iter() {
                if persona.principal_id == persona_r.principal_id {
                    return to_error_response("Persona already exists");
                }
            }
            let mut created_persona = persona_request_to_persona(persona_r.clone());
            personas.push(created_persona.clone());

            match PersonaRepo::store_personas(personas) {
                Some(t) => {
                    PrincipalIndex::store_principal(created_persona.principal_id_hash);
                    to_success_response(account_to_account_response(t.clone()))
                }

                None => to_error_response("Unable to store persona.")
            }
        }
        None => to_error_response("Unable to find Account.")
    }
}

pub fn update_persona(request: HTTPPersonaUpdateRequest) -> HttpResponse<AccountResponse> { //TODO needs to be refactored
    match PersonaRepo::get_persona(request.principal_id.clone()) {
        Some(mut persona) => {
            if !request.application.is_none() {
                persona.application = request.application.clone();
            }
            if !request.application_user_name.is_none() {
                persona.application_user_name = request.application_user_name.clone();
            }
            if request.anchor.is_some() {
                persona.anchor = request.anchor.clone();
            }
            let mut personas: Vec<Persona> = PersonaRepo::get_personas()
                .unwrap()
                .into_iter()
                .filter(|l| l.principal_id != request.principal_id)
                .collect();
            personas.push(persona.clone());
            match PersonaRepo::store_personas(personas) {
                Some(l) => {
                    to_success_response(account_to_account_response(l))
                }
                None => to_error_response("Unable to update Persona.")
            }
        }
        None => to_error_response("Unable to find Persona.")
    }
}

pub fn read_personas() -> HttpResponse<Vec<PersonaResponse>> {
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
