use crate::http::requests::{AccountRR, PersonaRequest, PersonaResponse};
use crate::mapper::account_mapper::account_to_account_response;
use crate::mapper::persona_mapper::{persona_request_to_persona, persona_to_persona_response};
use crate::repository::repo::{Persona, PersonaRepo, PrincipalIndex};
use crate::requests::HTTPPersonaUpdateRequest;
use crate::response_mapper::{HttpResponse, to_error_response, to_success_response};

pub fn create_persona(persona_r: PersonaRequest) -> HttpResponse<AccountRR> {
    match PersonaRepo::get_personas()
    {
        Some(mut personas) => {
            for persona in personas.iter() {
                if persona.principal_id == persona_r.principal_id {
                    return to_error_response("Persona already exists");
                }
            }
            let updated_persona = persona_request_to_persona(persona_r.clone());
            personas.push(updated_persona.clone());

            match PersonaRepo::store_personas(personas) {
                Some(t) => {
                    PrincipalIndex::store_principal(updated_persona.principal_id_hash);
                    to_success_response(account_to_account_response(t.clone()))
                }

                None => to_error_response("Unable to store persona.")
            }
        }
        None => to_error_response("Unable to find Account.")
    }
}

pub fn update_persona(request: HTTPPersonaUpdateRequest) -> HttpResponse<AccountRR> { //TODO needs to be refactored
    match PersonaRepo::get_persona(request.principal_id.clone()) {
        Some(mut persona) => {
            if request.name.is_some() {
                persona.name = request.name.clone().unwrap();
            }
            if !request.is_root.is_none() {
                persona.is_root = request.is_root.unwrap();
            }
            if !request.is_seed_phrase_copied.is_none() {
                persona.is_seed_phrase_copied = request.is_seed_phrase_copied.unwrap();
            }
            if !request.is_ii_anchor.is_none() {
                persona.is_ii_anchor = request.is_ii_anchor.unwrap();
            }
            if request.anchor.is_some() {
                persona.anchor = request.anchor.clone().unwrap();
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
