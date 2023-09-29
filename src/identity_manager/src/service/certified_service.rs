use ic_cdk::api::set_certified_data;
use ic_certified_map::{AsHashTree, RbTree};
use std::cell::RefCell;
use ic_cdk::export::{
    candid::CandidType,
};
use serde::Serialize;

thread_local! {
  pub static TREE: RefCell<RbTree<String, Vec<u8>>> = RefCell::new(RbTree::new());
}


#[derive(CandidType)]
pub struct CertifiedResponse {
    pub response: String,
    pub certificate: Vec<u8>,
    pub witness: Vec<u8>,
}

pub fn update_certify_keys(key: String, principal: String) -> String {
    TREE.with(|k| {
        let mut keys = k.borrow_mut();
        let b = hex::decode(sha256::digest(principal)).unwrap();
        keys.insert(key.clone(), b);
        set_certified_data(&keys.root_hash());
        key
    })
}

pub fn remove_certify_keys(key: String) {
    TREE.with(|k| {
        let mut keys = k.borrow_mut();
        keys.delete(key.as_ref());
        set_certified_data(&keys.root_hash());
    })
}

pub fn get_witness(key: String) -> anyhow::Result<Vec<u8>> {
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

#[test]
fn sub_account_test() {}