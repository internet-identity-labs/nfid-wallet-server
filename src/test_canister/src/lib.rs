use ic_cdk_macros::{query, update};
use std::cell::RefCell;
use ic_cdk::caller;
use ic_cdk::export::candid::{candid_method, export_service};


thread_local! {
    static ORIGIN_STORAGE: RefCell<Vec<String>> = RefCell::new(Default::default());
}



#[update]
#[candid_method(update)]
async fn get_trusted_origins() -> Vec<String> {
    ORIGIN_STORAGE.with(|storage| {
         storage.borrow().clone()
    })
}


#[update]
#[candid_method(update)]
async fn update_trusted_origins(a: Vec<String>) -> Vec<String> {
    ORIGIN_STORAGE.with(|storage| {
         storage.replace(a);
        storage.borrow().clone()
    })
}

#[query]
#[candid_method(query)]
async fn get_principal() -> String {
    caller().to_text()
}


export_service!();


#[ic_cdk_macros::query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    __export_service()
}