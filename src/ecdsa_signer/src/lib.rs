use std::cell::RefCell;
use std::collections::HashMap;
use std::str;

use candid::{candid_method, export_service};
use ic_cdk::{call, caller, id, storage, trap};
use ic_cdk::api::call::CallResult;
use ic_cdk::api::management_canister::main::CanisterStatusResponse;
use ic_cdk::api::set_certified_data;
use ic_cdk::export::{
    candid::CandidType,
    serde::{Deserialize, Serialize},
};
use ic_certified_map::{AsHashTree, RbTree};
use ic_cdk::export::candid::Principal;
use ic_cdk_macros::*;

#[derive(CandidType, Serialize, Debug, Deserialize)]
pub struct KeyPair {
    pub public_key: String,
    pub private_key_encrypted: String,
}

#[derive(CandidType, Serialize, Debug)]
pub struct KeyPairResponse {
    pub key_pair: Option<KeyPair>,
    pub princ: String,
}

#[derive(CandidType, Serialize, Debug)]
pub struct KeyPairObject {
    pub key_pair: KeyPair,
    pub created_date: u64,
}

#[derive(CandidType)]
struct CertifiedKeyPairResponse {
    response: KeyPairResponse,
    certificate: Vec<u8>,
    witness: Vec<u8>,
}

#[derive(Clone, Copy, Debug, CandidType, Deserialize, PartialEq, Serialize, Hash)]
pub enum StorageVariant {
    #[serde(rename = "ETH")]
    ETH,
    #[serde(rename = "BTC")]
    BTC,
    #[serde(rename = "II")]
    II,
}


thread_local! {
    static CONFIG: RefCell<Conf> = RefCell::new( Conf {
        controllers: Default::default(),
        storage: None,
        im_canister: None
    });
    static ECDSA_KEYS: RefCell<HashMap<String,KeyPairObject>> = RefCell::new(HashMap::new());
    static TREE: RefCell<RbTree<String, Vec<u8>>> = RefCell::new(RbTree::new());
}

#[query]
async fn get_kp_certified(key: String) -> CertifiedKeyPairResponse {
    trap_if_not_authenticated_admin();
    let witness = match get_count_witness(key.clone()) {
        Ok(tree) => tree,
        Err(_) => {
            Vec::default()
        }
    };

    let response = ECDSA_KEYS.with(|keys| {
        match keys.borrow().get(&key) {
            None => {
                KeyPairResponse { key_pair: None, princ: key.clone() }
            }
            Some(kp) => {
                let response = KeyPairResponse {
                    key_pair: Some(KeyPair {
                        public_key: kp.key_pair.public_key.clone(),
                        private_key_encrypted: kp.key_pair.private_key_encrypted.clone(),
                    }),
                    princ: key,
                };
                response
            }
        }
    });

    let certificate = ic_cdk::api::data_certificate().expect("No data certificate available");

    CertifiedKeyPairResponse {
        response,
        certificate,
        witness,
    }
}

#[derive(CandidType, Deserialize, Clone, Debug, Hash, PartialEq)]
pub struct Conf {
    pub controllers: Option<Vec<Principal>>,
    pub storage: Option<StorageVariant>,
    pub im_canister: Option<String>,
}

#[init]
async fn init(conf: Option<Conf>) -> () {
    match conf {
        Some(conf) => {
            CONFIG.with(|storage| {
                storage.replace(conf);
            });
        }
        _ => {}
    };
}

#[update]
async fn reconfig(conf: Conf) -> () {
    trap_if_not_authenticated_admin();
    CONFIG.with(|storage| {
        storage.replace(conf);
    })
}


#[derive(CandidType, Debug, Clone, Deserialize)]
pub struct CanisterIdRequest {
    #[serde(rename = "canister_id")]
    pub canister_id: Principal,
}

#[update]
#[candid_method(update)]
async fn sync_controllers() -> Vec<String> {
    let res: CallResult<(CanisterStatusResponse, )> = call(
        Principal::management_canister(),
        "canister_status",
        (CanisterIdRequest {
            canister_id: id(),
        }, ),
    ).await;

    let controllers = res.unwrap().0.settings.controllers;
    CONFIG.with(|c| c.borrow_mut().controllers.replace(controllers.clone()));
    controllers.iter().map(|x| x.to_text()).collect()
}

#[query]
#[candid_method(query)]
async fn get_public_key(root_principal: String) -> Option<String> {
    ECDSA_KEYS.with(|keys| {
        match keys.borrow().get(&root_principal) {
            None => {
                None
            }
            Some(kp) => {
                Some(kp.key_pair.public_key.clone())
            }
        }
    })
}

#[update]
#[candid_method(query)]
async fn get_kp() -> KeyPairResponse {
    let key = match get_root_id().await {
        None => { trap("Unauthorised") }
        Some(k) => { k }
    };
    ECDSA_KEYS.with(|keys| {
        match keys.borrow().get(&key) {
            None => {
                KeyPairResponse { key_pair: None, princ: key }
            }
            Some(kp) => {
                let response = KeyPairResponse {
                    key_pair: Some(KeyPair {
                        public_key: kp.key_pair.public_key.clone(),
                        private_key_encrypted: kp.key_pair.private_key_encrypted.clone(),
                    }),
                    princ: key,
                };
                response
            }
        }
    })
}


#[update]
#[candid_method(update)]
async fn add_kp(kp: KeyPair) {
    let key = match get_root_id().await {
        None => { trap("Unauthorised") }
        Some(k) => { k }
    };
    ECDSA_KEYS.with(|k| {
        let mut keys = k.borrow_mut();
        if keys.contains_key(&key) {
            trap(&format!("Already registered {}", key))
        }
        let kkp = KeyPairObject {
            key_pair: KeyPair {
                public_key: kp.public_key.clone(),
                private_key_encrypted: kp.private_key_encrypted.clone(),
            },
            created_date: ic_cdk::api::time(),
        };
        keys.insert(key.clone(), kkp);
        update_certify_keys(key, KeyPair {
            public_key: kp.public_key,
            private_key_encrypted: kp.private_key_encrypted,
        });
    })
}

#[query]
#[candid_method(query)]
async fn get_principal(payload: Option<String>) -> (String, Option<String>) {
    let principal = ic_cdk::caller().to_text();
    (principal, payload)
}

#[pre_upgrade]
fn pre_upgrade() {
    let conf: Conf = CONFIG.with(|c| {
        return c.borrow_mut().clone();
    });
    let principal_key_pairs = ECDSA_KEYS.with(|k| {
        let pkp: Vec<PrincipalKP> = k.borrow_mut()
            .iter()
            .map(|a| PrincipalKP {
                public_key: a.1.key_pair.public_key.clone(),
                private_key: a.1.key_pair.private_key_encrypted.clone(),
                created_date: a.1.created_date.clone(),
                principal: a.0.clone(),
            })
            .collect();
        pkp
    });

    let pre_upgrade_data = PersistedData { conf: Some(conf), keys: Some(principal_key_pairs) };
    match storage::stable_save((pre_upgrade_data, 0)) {
        Ok(_) => (),
        Err(_) => trap(&format!("Failed to pre_upgrade"))
    }
}

#[query]
async fn get_all_json(from: u32, mut to: u32) -> String {
    trap_if_not_authenticated_admin();
    let mut principal_key_pairs = ECDSA_KEYS.with(|k| {
        let pkp: Vec<PrincipalKP> = k.borrow_mut()
            .iter()
            .map(|a| PrincipalKP {
                public_key: a.1.key_pair.public_key.clone(),
                private_key: a.1.key_pair.private_key_encrypted.clone(),
                created_date: a.1.created_date.clone(),
                principal: a.0.clone(),
            })
            .collect();
        pkp
    });
    principal_key_pairs.sort_by(|a, b| a.created_date.cmp(&b.created_date));
    let len = principal_key_pairs.len() as u32;
    if to > len {
        to = len;
    }
    let resp = &principal_key_pairs[from as usize..to as usize];
    return serde_json::to_string(&resp).unwrap();
}

#[query]
async fn count() -> u64 {
    trap_if_not_authenticated_admin();
    ECDSA_KEYS.with(|k| {
        k.borrow().len() as u64
    })
}

fn trap_if_not_authenticated_admin() {
    let princ = caller();
    match CONFIG.with(|c| c.borrow_mut().controllers.clone())
    {
        None => {
            trap("Unauthorised")
        }
        Some(controllers) => {
            if !controllers.contains(&princ) {
                trap("Unauthorised")
            }
        }
    }
}

async fn get_root_id() -> Option<String> {
    match CONFIG.with(|c| c.borrow_mut().im_canister.clone()) {
        None => {
            Some(caller().to_text())  //DONE FOR TESTING PURPOSES
        }
        Some(canister) => {
            let princ = caller();
            let im_canister = Principal::from_text(canister).unwrap();

            let res: Option<String> = match call(im_canister, "get_root_by_principal", (princ.to_text(), 0)).await {
                Ok((res, )) => res,
                Err((_, err)) => trap(&format!("failed to request IM: {}", err)),
            };
            res
        }
    }
}


#[derive(Clone, Debug, Deserialize, CandidType)]
pub struct PersistedData {
    pub conf: Option<Conf>,
    pub keys: Option<Vec<PrincipalKP>>,
}

#[derive(CandidType, Deserialize, Clone, Debug, Hash, PartialEq, Serialize)]
pub struct PrincipalKP {
    pub public_key: String,
    pub private_key: String,
    pub principal: String,
    pub created_date: u64,
}

#[post_upgrade]
fn post_upgrade() {
    match storage::stable_restore() {
        Ok(store) => {
            let (post_data, _a): (PersistedData, i32) = store;
            if post_data.conf.is_some() {
                CONFIG.with(|storage| {
                    storage.replace(post_data.conf.clone().unwrap());
                });
            };
            if post_data.keys.is_some() {
                ECDSA_KEYS.with(|storage| {
                    let mut kpp = HashMap::default();
                    let mut tree: RbTree<String, Vec<u8>> = RbTree::new();
                    for x in post_data.keys.unwrap().into_iter() {
                        kpp.insert(x.principal.clone(),
                                   KeyPairObject {
                                       key_pair: KeyPair {
                                           public_key: x.public_key.clone(),
                                           private_key_encrypted: x.private_key.clone(),
                                       },
                                       created_date: x.created_date,
                                   },
                        );
                        let new_owned_string = x.public_key.clone() + &x.private_key;
                        let b = hex::decode(sha256::digest(new_owned_string)).unwrap();
                        tree.insert(x.principal, b);
                    };
                    TREE.with(|tr| {
                        set_certified_data(&tree.root_hash());
                        tr.replace(tree);
                    });
                    storage.replace(kpp);
                });
            }
        }
        Err(message) => trap(message.as_str())
    }
}


fn get_count_witness(key: String) -> anyhow::Result<Vec<u8>> {
    TREE.with(|tree| {
        let tree = tree.borrow();
        let mut witness = vec![];
        let mut witness_serializer = serde_cbor::Serializer::new(&mut witness);

        witness_serializer.self_describe()?;

        tree.witness(key.as_bytes())
            .serialize(&mut witness_serializer)
            .unwrap();

        Ok(witness)
    })
}

fn update_certify_keys(key: String, kp: KeyPair) -> String {
    TREE.with(|k| {
        let mut keys = k.borrow_mut();
        let new_owned_string = kp.public_key.clone() + &kp.private_key_encrypted;
        let b = hex::decode(sha256::digest(new_owned_string)).unwrap();
        keys.insert(key.clone(), b);
        set_certified_data(&keys.root_hash());
        key
    })
}
export_service!();

#[ic_cdk_macros::query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    __export_service()
}

#[test]
fn sub_account_test() {}