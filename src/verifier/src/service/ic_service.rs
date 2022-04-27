use ic_cdk::export::Principal;

pub fn get_caller() -> Principal {
    ic_cdk::api::caller()
}


