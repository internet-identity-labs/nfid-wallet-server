use crate::http::requests::{PersonaRequest, PersonaResponse};
use crate::repository::repo::{calculate_hash, Persona};

pub fn persona_to_persona_response(persona: Persona) -> PersonaResponse {
    PersonaResponse {
        name: persona.name,
        is_root: persona.is_root,
        is_seed_phrase_copied: persona.is_seed_phrase_copied,
        is_ii_anchor: persona.is_ii_anchor,
        anchor: persona.anchor,
        principal_id: persona.principal_id,
    }
}

pub fn persona_request_to_persona(persona_request: PersonaRequest) -> Persona {
    let required_id = persona_request.principal_id_origin.clone();
    let hashed_persona_principal = calculate_hash(required_id.clone().as_str());
    Persona {
        name: persona_request.name,
        is_root: persona_request.is_root,
        is_seed_phrase_copied: persona_request.is_seed_phrase_copied,
        is_ii_anchor: persona_request.is_ii_anchor,
        anchor: persona_request.anchor,
        principal_id_hash: hashed_persona_principal,
        principal_id: persona_request.principal_id,
    }
}
