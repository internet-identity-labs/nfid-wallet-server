use crate::service::principle_service::get_principal;
use crate::repository::repo::{Account, AccountRepo, Persona, PersonaRepo, PrincipalIndex};
use crate::requests::HTTPPersonaUpdateRequest;
use crate::response_mapper::{HttpResponse, to_error_response, to_success_response};

pub fn create_persona(persona: Persona) -> HttpResponse<Account> {
    let princ = &ic_cdk::api::caller().to_text();
    let root_id = get_principal(princ);
    match PersonaRepo::get_personas(root_id)
    {
        Some(mut personas) => {
            let required_id = persona.principal_id.clone();
            for persona in personas.iter() {
                if persona.principal_id == required_id {
                    return to_error_response("Persona already exists");
                }
            }
            personas.push(persona.clone());
            PersonaRepo::store_personas(get_principal(princ), personas);
            PrincipalIndex::store_principal(persona.principal_id.as_str(), root_id);

            match AccountRepo::get_account(persona.principal_id.as_str()) {  //todo rethink
                Some(t) => { to_success_response(t.clone()) }

                None => to_error_response("Unable to store persona.")
            }
        }
        None => to_error_response("Unable to find Account.")
    }
}

pub fn update_persona(request: HTTPPersonaUpdateRequest) -> HttpResponse<Account> { //TODO needs to be refactored
    let princ = &ic_cdk::api::caller().to_text();
    match PersonaRepo::get_persona(&request.principal_id) {
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
            let mut personas: Vec<Persona> = PersonaRepo::get_personas(get_principal(princ))
                .unwrap()
                .into_iter()
                .filter(|l| l.principal_id != request.principal_id)
                .collect();
            personas.push(persona.clone());
            match PersonaRepo::store_personas(get_principal(princ), personas) {
                Some(mut l) => {
                    to_success_response(l) }
                None => to_error_response("Unable to store Persona.")
            }
        }
        None => to_error_response("Unable to find Persona.")
    }
}

pub fn read_personas() -> HttpResponse<Vec<Persona>> {
    let princ = &ic_cdk::api::caller().to_text();
    match PersonaRepo::get_personas(get_principal(princ)) {
        Some(acc) => {
            to_success_response(acc)
        }
        None => to_error_response("Unable to find Account.")
    }
}
