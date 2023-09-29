use ic_cdk_macros::{query, update};
use std::cell::RefCell;
use ic_cdk::api::set_certified_data;
use ic_cdk::caller;
use ic_cdk::export::candid::{candid_method, export_service, CandidType};
use ic_cdk::export::{
    serde::{Serialize},
};
use ic_cdk_macros::*;
use ic_certified_map::{AsHashTree, RbTree};


#[derive(CandidType)]
struct CertifiedResponse {
    response: Vec<String>,
    certificate: Vec<u8>,
    witness: Vec<u8>,
}

thread_local! {
    static ORIGIN_STORAGE_CERTIFIED: RefCell<Vec<String>> = RefCell::new(Default::default());
    static ORIGIN_STORAGE_RAW: RefCell<Vec<String>> = RefCell::new(Default::default());
    static TREE: RefCell<RbTree<String, Vec<u8>>> = RefCell::new(RbTree::new());
}


#[update]
#[candid_method(update)]
async fn get_trusted_origins() -> Vec<String> {
    ORIGIN_STORAGE_RAW.with(|storage| {
        storage.borrow().clone()
    })
}

#[query]
#[candid_method(query)]
async fn get_trusted_origins_certified() -> CertifiedResponse {
    let witness = match get_count_witness("origins".to_string()) {
        Ok(tree) => tree,
        Err(_) => {
            Vec::default()
        }
    };
    let origins = ORIGIN_STORAGE_CERTIFIED.with(|storage| {
        storage.borrow().clone()
    });

    let certificate = ic_cdk::api::data_certificate().expect("No data certificate available");

    CertifiedResponse {
        response: origins,
        certificate,
        witness,
    }
}


#[update]
#[candid_method(update)]
async fn update_trusted_origins(a: Vec<String>) -> Vec<String> {
    update_certify_keys("origins".to_string(), a.clone());
    ORIGIN_STORAGE_CERTIFIED.with(|storage| {
        storage.replace(a);
        storage.borrow().clone()
    })
}


#[update]
#[candid_method(update)]
async fn update_trusted_origins_raw(a: Vec<String>) -> Vec<String> {
    ORIGIN_STORAGE_RAW.with(|storage| {
        storage.replace(a);
        storage.borrow().clone()
    })
}

#[query]
#[candid_method(query)]
async fn get_principal() -> String {
    caller().to_text()
}


#[post_upgrade]
async fn post_upgrade() {
    let a: Vec<String> = vec!["http://localhost:4200".to_string(),
                              "nfid.one".to_string(),
                              "https://wzkxy-vyaaa-aaaaj-qab3q-cai.ic0.app".to_string(),
                              "https://playground-dev.nfid.one".to_string(), ];
    update_trusted_origins(a.clone()).await;
    update_trusted_origins_raw(a).await;
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

export_service!();

fn update_certify_keys(key: String, origins: Vec<String>) -> String {
    TREE.with(|k| {
        let mut keys = k.borrow_mut();
        let concatenated_string: String = origins.join("");
        let b = hex::decode(sha256::digest(concatenated_string)).unwrap();
        keys.insert(key.clone(), b);
        set_certified_data(&keys.root_hash());
        key
    })
}

#[ic_cdk_macros::query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    __export_service()
}


#[test]
fn build() {}