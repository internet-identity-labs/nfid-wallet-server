use crate::http::requests::{PersonaVariant};
use crate::repository::persona_repo::Persona;
use crate::repository::repo::BasicEntity;
use crate::requests::{PersonaII, PersonaNFID};

pub fn persona_to_persona_response(persona: Persona) -> PersonaVariant {
    match persona.anchor {
        None => {
            PersonaVariant::NfidPersona(PersonaNFID {
                domain: persona.domain,
                persona_id: persona.persona_id.unwrap(),
            })
        }
        Some(_) => {
            PersonaVariant::IiPersona(PersonaII {
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
                base_fields: BasicEntity::new(),
            }
        }
        PersonaVariant::NfidPersona(nfid) => {
            Persona {
                anchor: None,
                domain: nfid.domain,
                persona_id: Option::from(nfid.persona_id),
                base_fields: BasicEntity::new(),
            }
        }
    }
}
