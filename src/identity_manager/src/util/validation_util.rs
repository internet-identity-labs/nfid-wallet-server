use lazy_static::lazy_static;
use regex::Regex;
use crate::PersonaVariant;

pub fn validate_name(name: &str) -> bool {
    lazy_static! {
    static ref RE: Regex  = Regex::new(r"^[0-9a-zA-Z][0-9a-zA-Z _]{2,30}$").unwrap();
        }
    RE.is_match(name)
}


pub fn validate_frontend_length(persona_request: &PersonaVariant) -> bool {
    const FRONTEND_HOSTNAME_LIMIT: usize = 255;

    match persona_request {
        PersonaVariant::IiPersona(ii) => {
            ii.domain.len() < FRONTEND_HOSTNAME_LIMIT
        }
        PersonaVariant::NfidPersona(nfid) => {
            (nfid.persona_id.len() + nfid.domain.len()) < FRONTEND_HOSTNAME_LIMIT
        }
    }
}
