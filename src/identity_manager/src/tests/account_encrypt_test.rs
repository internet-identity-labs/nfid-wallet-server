use std::collections::HashSet;
use crate::repository::access_point_repo::AccessPoint;
use crate::repository::account_repo::Account;
use crate::repository::encrypt::account_encrypt::{decrypt_access_point, decrypt_account, decrypt_persona, encrypt_access_point, encrypt_account, encrypt_persona};
use crate::repository::persona_repo::Persona;
use crate::tests::test_util::init_config;

#[test]
fn encrypt_decrypt_test() {
    init_config();
    let ap = AccessPoint {
        pub_key: "".to_string(),
        last_used: "".to_string(),
        make: "".to_string(),
        model: "".to_string(),
        browser: "".to_string(),
        name: "".to_string(),
        base_fields: Default::default(),
    };
    let mut apv = HashSet::new();
    apv.insert(ap.clone());
    let acc = Account {
        anchor: 0,
        principal_id: "".to_string(),
        name: "".to_string(),
        phone_number: "".to_string(),
        personas: vec![],
        access_points: apv,
        base_fields: Default::default(),
    };
    let encrypted = encrypt_account(acc.clone());
    assert_ne!(acc.name, encrypted.name);
    assert_ne!(acc.principal_id, encrypted.principal_id);
    assert_ne!(acc.phone_number, encrypted.phone_number);
    assert_ne!(acc.anchor.to_string(), encrypted.anchor);
    let decrypted = decrypt_account(encrypted);
    assert_eq!(acc.name, decrypted.name);
    assert_eq!(acc.principal_id, decrypted.principal_id);
    assert_eq!(acc.phone_number, decrypted.phone_number);
    assert_eq!(acc.anchor, decrypted.anchor);
    let dap = decrypted.access_points.get(&ap).unwrap();
    assert_eq!(ap.pub_key, dap.pub_key);
    assert_eq!(ap.last_used, dap.last_used);
    assert_eq!(ap.make, dap.make);
    assert_eq!(ap.model, dap.model);
    assert_eq!(ap.browser, dap.browser);
    assert_eq!(ap.name, dap.name);
}

#[test]
fn encrypt_decrypt_persona() {
    init_config();
    let persona = Persona {
        anchor: Option::from(1),
        domain: "domain".to_string(),
        persona_id: Option::from("id".to_string()),
        base_fields: Default::default(),
    };
    let encrypted = encrypt_persona(persona.clone());
    assert_ne!(persona.anchor.unwrap().to_string(), encrypted.anchor.clone().unwrap().to_string());
    assert_ne!(persona.domain, encrypted.domain);
    assert_ne!(persona.persona_id.clone().unwrap(), encrypted.persona_id.clone().unwrap());
    let decrypted = decrypt_persona(encrypted);
    assert_eq!(persona.anchor.unwrap().to_string(), decrypted.anchor.unwrap().to_string());
    assert_eq!(persona.domain, decrypted.domain);
    assert_eq!(persona.persona_id.clone().unwrap(), decrypted.persona_id.unwrap());
}

#[test]
fn encrypt_decrypt_nullable_persona() {
    init_config();
    let persona = Persona {
        anchor: None,
        domain: "domain".to_string(),
        persona_id: None,
        base_fields: Default::default(),
    };
    let encrypted = encrypt_persona(persona.clone());
    assert_eq!(persona.anchor.unwrap_or(0).to_string(), encrypted.anchor.clone().unwrap_or("0".to_string()).to_string());
    assert_ne!(persona.domain, encrypted.domain);
    assert_eq!(persona.persona_id.clone().unwrap_or("aa".to_string()), encrypted.persona_id.clone().unwrap_or("aa".to_string()));
    let decrypted = decrypt_persona(encrypted);
    assert_eq!(persona.anchor.unwrap_or(0).to_string(), decrypted.anchor.unwrap_or(0).to_string());
    assert_eq!(persona.domain, decrypted.domain);
    assert_eq!(persona.persona_id.clone().unwrap_or("aa".to_string()), decrypted.persona_id.clone().unwrap_or("aa".to_string()));
}

#[test]
fn encrypt_decrypt_access_point() {
    init_config();
    let ap = AccessPoint {
        pub_key: "".to_string(),
        last_used: "".to_string(),
        make: "".to_string(),
        model: "".to_string(),
        browser: "".to_string(),
        name: "".to_string(),
        base_fields: Default::default(),
    };
    let encrypted = encrypt_access_point(ap.clone());
    assert_ne!(ap.pub_key, encrypted.pub_key);
    assert_ne!(ap.last_used, encrypted.last_used);
    assert_ne!(ap.make, encrypted.make);
    assert_ne!(ap.model, encrypted.model);
    assert_ne!(ap.browser, encrypted.browser);
    assert_ne!(ap.name, encrypted.name);
    let decrypted = decrypt_access_point(encrypted);
    assert_eq!(ap.pub_key, decrypted.pub_key);
    assert_eq!(ap.last_used, decrypted.last_used);
    assert_eq!(ap.make, decrypted.make);
    assert_eq!(ap.model, decrypted.model);
    assert_eq!(ap.browser, decrypted.browser);
    assert_eq!(ap.name, decrypted.name);
}
