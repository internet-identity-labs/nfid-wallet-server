use crate::PersonaVariant;
use crate::requests::{PersonaIIResponse, PersonaNFIDResponse};
use crate::util::validation_util::{validate_frontend_length, validate_name};

#[test]
fn validate_name_test() {
    assert!(validate_name("John Doe"));
    assert!(validate_name("John_Doe"));
    assert!(validate_name("123John_Doe123"));
    assert!(!validate_name("1234567891011121312345678910111213"));
    assert!(!validate_name("Jo"));
    assert!(!validate_name("John *"));
}

#[test]
fn validate_persona_util_test() {
    let p_ii = PersonaVariant::IiPersona(PersonaIIResponse {
        anchor: 123456,
        domain: "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".to_string(),
    });
    assert!(!validate_frontend_length(p_ii));
    let p_nfid = PersonaVariant::NfidPersona(PersonaNFIDResponse {
        domain: "Domain".to_string(),
        persona_id: "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".to_string(),
    });
    assert!(!validate_frontend_length(p_nfid));
    let p_ii_1 = PersonaVariant::IiPersona(PersonaIIResponse {
        anchor: 123456,
        domain: "domain".to_string(),
    });
    assert!(validate_frontend_length(p_ii_1));
    let p_nfid_1 = PersonaVariant::NfidPersona(PersonaNFIDResponse {
        domain: "Domain".to_string(),
        persona_id: "Persona".to_string(),
    });
    assert!(validate_frontend_length(p_nfid_1));
}


