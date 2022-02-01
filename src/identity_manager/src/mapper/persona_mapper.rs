use crate::http::requests::{PersonaVariant};
use crate::repository::repo::{Persona};
use crate::requests::{PersonaIIResponse, PersonaNFIDResponse};

pub fn persona_to_persona_response(persona: Persona) -> PersonaVariant {
    match persona.anchor {
        None => {
            PersonaVariant::NfidPersona(PersonaNFIDResponse {
                domain: persona.domain,
                persona_id: persona.persona_id.unwrap(),
            })
        }
        Some(_) => {
            PersonaVariant::IiPersona(PersonaIIResponse {
                domain: persona.domain,
                anchor: persona.anchor.unwrap(),
            })
        }
    }
}

pub fn persona_request_to_persona(persona_request: PersonaVariant) -> Persona {
    match persona_request {
        PersonaVariant::IiPersona(ii) => {
            Persona {
                anchor: Option::from(ii.anchor),
                domain: ii.domain,
                persona_id: None,
            }
        }
        PersonaVariant::NfidPersona(nfid) => {
            Persona {
                anchor: None,
                domain: nfid.domain,
                persona_id: Option::from(nfid.persona_id),
            }
        }
    }
}
