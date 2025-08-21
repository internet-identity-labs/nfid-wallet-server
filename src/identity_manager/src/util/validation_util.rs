use crate::requests::PersonaRequest;
use lazy_static::lazy_static;
use regex::Regex;

pub fn validate_name(name: &str) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^[0-9a-zA-Z][0-9a-zA-Z _]{2,30}$")
            .expect("Failed to compile regular expression for name validation.");
    }
    RE.is_match(name)
}

pub fn validate_frontend_length(persona_request: &PersonaRequest) -> bool {
    const FRONTEND_HOSTNAME_LIMIT: usize = 255;
    persona_request.domain.len() < FRONTEND_HOSTNAME_LIMIT
}
