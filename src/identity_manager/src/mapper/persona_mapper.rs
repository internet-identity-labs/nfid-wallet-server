use crate::repository::persona_repo::Persona;
use crate::repository::repo::BasicEntity;
use crate::requests::{PersonaRequest, PersonaResponse};

pub fn persona_to_persona_response(persona: Persona) -> PersonaResponse {
    PersonaResponse {
        domain: persona.domain,
        persona_id: persona.persona_id,
        persona_name: persona.persona_name.unwrap_or("".to_string()),
    }
}

pub fn persona_request_to_persona(persona_request: PersonaRequest) -> Persona {
    Persona {
        domain: persona_request.domain,
        persona_name: Some(persona_request.persona_name),
        persona_id: persona_request.persona_id,
        base_fields: BasicEntity::new(),
        domain_certified: None,
    }
}
