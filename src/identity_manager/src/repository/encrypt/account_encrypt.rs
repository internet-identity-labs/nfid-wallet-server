extern crate base64;

use std::io::Cursor;
use std::option::Option;
use magic_crypt::{MagicCrypt256, MagicCryptTrait, new_magic_crypt};
use magic_crypt::generic_array::typenum::U256;
use crate::{ConfigurationRepo};
use crate::repository::account_repo::Account;
use crate::repository::encrypt::encrypted_repo::{EncryptedAccount, EncryptedPersona};
use crate::repository::persona_repo::Persona;


pub fn encrypt_account(account: Account) -> EncryptedAccount {
    let key = get_key();
    let crypt = new_magic_crypt!(key.clone(), 256);
    EncryptedAccount {
        anchor: encrypt_string(&crypt, account.anchor.to_string()),
        principal_id: encrypt_string(&crypt, account.principal_id),
        name: encrypt_string(&crypt, account.name),
        phone_number: encrypt_string(&crypt, account.phone_number),
        personas: account.personas.into_iter().map(|l| encrypt_persona(l)).collect(),
        base_fields: account.base_fields,
    }
}

pub fn decrypt_account(account: EncryptedAccount) -> Account {
    let key = get_key();
    let crypt = new_magic_crypt!(key.clone(), 256);
    Account {
        anchor: decrypt_number(&crypt, account.anchor),
        principal_id: crypt.decrypt_base64_to_string(account.principal_id).unwrap(),
        name: crypt.decrypt_base64_to_string(account.name).unwrap(),
        phone_number: crypt.decrypt_base64_to_string(account.phone_number).unwrap(),
        personas: account.personas.into_iter().map(|l| decrypt_persona(l)).collect(),
        base_fields: account.base_fields,
    }
}

pub fn decrypt_phone_number(account: EncryptedAccount) -> String {
    let key = get_key();
    let crypt = new_magic_crypt!(key.clone(), 256);
    crypt.decrypt_base64_to_string(account.phone_number).unwrap()
}

pub fn encrypt_persona(persona: Persona) -> EncryptedPersona {
    let key = get_key();
    let crypt = new_magic_crypt!(key.clone(), 256);
    EncryptedPersona {
        anchor: encrypt_optional_number(&crypt, persona.anchor),
        domain: encrypt_string(&crypt, persona.domain.to_string()),
        persona_id: encrypt_optional(&crypt, persona.persona_id),
        base_fields: persona.base_fields,
    }
}

pub fn decrypt_persona(persona: EncryptedPersona) -> Persona {
    let key = get_key();
    let crypt = new_magic_crypt!(key.clone(), 256);
    Persona {
        anchor: decrypt_optional_number(&crypt, persona.anchor, &decrypt_number),
        domain: crypt.decrypt_base64_to_string(persona.domain).unwrap(),
        persona_id: decrypt_optional(&crypt, persona.persona_id, &decrypt_as_string),
        base_fields: persona.base_fields,
    }
}

pub fn encrypt(str: String) -> String {
    let key = get_key();
    let crypt = new_magic_crypt!(key.clone(), 256);
    encrypt_string(&crypt, str)
}

fn encrypt_string(crypt: &MagicCrypt256, str: String) -> String {
    let mut reader = Cursor::new(str);
    let mut writer = Vec::new();
    crypt.encrypt_reader_to_writer2::<U256>(&mut reader, &mut writer).unwrap();
    base64::encode(&writer)
}

fn decrypt_number(crypt: &MagicCrypt256, number: String) -> u64 {
    crypt.decrypt_base64_to_string(number).unwrap().parse().unwrap()
}

fn decrypt_as_string(crypt: &MagicCrypt256, number: String) -> String {
    crypt.decrypt_base64_to_string(number).unwrap()
}

fn encrypt_optional(crypt: &MagicCrypt256, opt: Option<String>) -> Option<String> {
    match opt {
        Some(aaa) => {
            Option::from(encrypt_string(&crypt, aaa))
        }
        None => { None }
    }
}

fn encrypt_optional_number(crypt: &MagicCrypt256, opt: Option<u64>) -> Option<String> {
    match opt {
        Some(aaa) => {
            Option::from(encrypt_string(&crypt, aaa.to_string()))
        }
        None => { None }
    }
}

fn decrypt_optional(crypt: &MagicCrypt256, opt: Option<String>, f: &dyn Fn(&MagicCrypt256, String) -> String) -> Option<String> {
    match opt {
        Some(aaa) => {
            Option::from(f(crypt, aaa))
        }
        None => { None }
    }
}

fn decrypt_optional_number(crypt: &MagicCrypt256, opt: Option<String>, f: &dyn Fn(&MagicCrypt256, String) -> u64) -> Option<u64> {
    match opt {
        Some(aaa) => {
            Option::from(f(crypt, aaa))
        }
        None => { None }
    }
}

fn get_key() -> String {
    std::str::from_utf8(ConfigurationRepo::get().key.as_slice()).unwrap().to_string()
}