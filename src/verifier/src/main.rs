use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::convert::TryInto;
use std::io::Cursor;
use std::option::Option;
use std::ptr::null;
use std::str;
use std::time::Duration;
use ic_cdk::export::candid::{CandidType, Deserialize};

use ic_cdk::{call, print, storage, trap};
use ic_cdk::export::Principal;
use ic_cdk_macros::{query, update};
use canister_api_macros::{log_error, replicate_account, admin};
use ic_cdk_macros::post_upgrade;
use ic_cdk_macros::pre_upgrade;

pub type Salt = [u8; 32];


#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ConfigurationRequest {
    pub identity_manager: String,
}

#[derive(Debug, Deserialize, CandidType, Clone)]
pub struct Configuration {
    pub identity_manager_canister_id: String,
}

#[update]
// #[admin]
async fn configure(request: ConfigurationRequest) -> () {
    let configuration = Configuration {
        identity_manager_canister_id: request.identity_manager,
    };
    ConfigurationRepo::save(configuration);
}

#[derive(Clone, Debug, Deserialize, CandidType, Hash, Eq, PartialEq)]
struct Certificate {
    client_principal: String,
    domain: String,
    phone_number_sha2: Option<String>,
}



const DEFAULT_TOKEN_TTL: Duration = Duration::from_secs(10);

thread_local! {
    static TOKEN_STORAGE: RefCell<HashMap<Salt, Certificate>> = RefCell::new(HashMap::new());
    static CERTIFICATE_STORAGE: RefCell<HashMap<String, Certificate>> = RefCell::new(HashMap::new());
}

#[query]
async fn ping() -> () {}

pub type Certs = BTreeMap<String, Certificate>;

#[query]
async fn is_owner_can(principal_id: String, token: String) -> bool {
    if principal_id.eq("atjar-wjnep-ewoeh-yrv6e-su7fu-hbsv6-vssff-abmsn-ppomw-m2pdi-dae") &&  //OWNER OF NFT PRINCIPAL_ID
        token.eq("test_token") { //TOKEN_ID
        return true;
    }
    return false;
}

#[query]
async fn get_ph_sha2(principal_id: String) -> String {
    return "+38".to_string();
}


#[update]
async fn post_certificate(domain: String) -> Salt {
    let t = ic_cdk::api::caller().to_text();

    let res: Vec<u8> = match call(Principal::management_canister(), "raw_rand", ()).await {
        Ok((res, )) => res,
        Err((_, err)) => trap(&format!("failed to get salt: {}", err)),
    };
    let salt: Salt = res[..].try_into().unwrap_or_else(|_| {
        trap(&format!(
            "expected raw randomness to be of length 32, got {}",
            res.len()
        ));
    });

    let cert = Certificate {
        client_principal: t,
        domain,
        phone_number_sha2: None,
    };

    TOKEN_STORAGE.with(|storage| {
        let mut st = storage.borrow_mut();
        let x = cert.client_principal.clone();
        st.insert(salt.clone(), cert.clone());
    });
    salt
}

#[query]
async fn is_owner(who: String) -> HttpResponse<bool> {
    CERTIFICATE_STORAGE.with(|storage| {
        let mut st = storage.borrow_mut();
        return match st.get(&who) {
            Some(_) => {
                to_success_response(true)
            }
            None => {
                to_success_response(false)
            }
        };
    })
}


pub fn to_success_response<T>(x: T) -> HttpResponse<T> {
    HttpResponse {
        data: Option::from(x),
        error: None,
        status_code: 200,
    }
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HttpResponse<T> {
    pub data: Option<T>,
    pub error: Option<String>,
    pub status_code: u16,
}

#[update]
async fn update_certificate(token: Salt) -> Option<Certificate> {
    let principal = ic_cdk::api::caller().to_text();
    let id = ConfigurationRepo::get().identity_manager_canister_id.clone();
    let a: HttpResponse<String> = match call(Principal::from_text(id).unwrap(), "get_phone_number_sha2", (principal.clone(), 0)).await
    {
        Ok((res, )) => res,
        Err((_, err)) => trap(&format!("failed: {}", err)),
    };
    match a.error {
        None => {}
        Some(error) => { trap(&format!("failed: {}", error)) }
    }
    TOKEN_STORAGE.with(|storage| {
        let mut st = storage.borrow_mut();
        match st.remove(&token) {
            Some(cert) => {
                let mut updated = cert.clone();
                updated.phone_number_sha2 = a.data;

                CERTIFICATE_STORAGE.with(|stable_storage| {
                    let mut cert_st = stable_storage.borrow_mut();
                    cert_st.insert(updated.client_principal.clone(), updated.clone());
                });
                Some(updated)
            }
            None => { None }
        }
    })
}


fn main() {}


pub struct ConfigurationRepo {}

impl ConfigurationRepo {
    //todo fix Principle not implement default!
    pub fn get() -> &'static Configuration {
        storage::get::<Option<Configuration>>().as_ref().unwrap()
    }

    pub fn exists() -> bool {
        storage::get::<Option<Configuration>>().is_some()
    }

    pub fn save(configuration: Configuration) -> () {
        storage::get_mut::<Option<Configuration>>().replace(configuration);
    }
}


#[pre_upgrade]
async fn pre_upgrade() {
    CERTIFICATE_STORAGE.with(|storage| {
        let mut st = storage.borrow_mut();
        let mut certs = HashSet::new();

        for cc in st.iter() {
            certs.insert(cc);
        }
        storage::stable_save((certs, 0));
    });
}

#[post_upgrade]
fn post_upgrade() {
    let poap: (HashSet<Certificate>, i32) = storage::stable_restore().unwrap();
    CERTIFICATE_STORAGE.with(|storage| {
        let mut st = storage.borrow_mut();
        for p in poap.0.iter() {
            let cert = p.to_owned();
            st.insert(cert.client_principal.clone(), cert);
        }
    });
}
