use crate::http::requests::{PersonaRequest, PersonaResponse};
use crate::repository::repo::{calculate_hash, Persona};

pub fn persona_to_persona_response(persona: Persona) -> PersonaResponse {
    PersonaResponse {
        anchor: persona.anchor,
        principal_id: persona.principal_id,
        application_user_name: persona.application_user_name,
        application: persona.application,
    }
}

pub fn persona_request_to_persona(persona_request: PersonaRequest) -> Persona {
    let required_id = persona_request.principal_id_origin.clone();
    let hashed_persona_principal = calculate_hash(required_id.clone().as_str());
    Persona {
        anchor: persona_request.anchor,
        principal_id_hash: hashed_persona_principal,
        principal_id: persona_request.principal_id,
        application_user_name: persona_request.application_user_name,
        application: persona_request.application,
    }
}
