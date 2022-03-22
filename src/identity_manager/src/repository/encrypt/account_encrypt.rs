extern crate base64;

use std::io::Cursor;
use std::option::Option;

use magic_crypt::{MagicCrypt256, MagicCryptTrait, new_magic_crypt};
use magic_crypt::generic_array::typenum::U256;

use crate::ConfigurationRepo;

pub fn encrypt(str: String) -> String {
    let key = get_key();
    let crypt = new_magic_crypt!(key.clone(), 256);
    encrypt_string(&crypt, str)
}

pub fn decrypt(str: String) -> String {
    let key = get_key();
    let crypt = new_magic_crypt!(key.clone(), 256);
    decrypt_as_string(&crypt, str)
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
