use ic_cdk::export::Principal;

#[cfg(test)]
pub fn get_caller() -> Principal {
    Principal::anonymous()
}

#[cfg(not(test))]
pub fn get_caller() -> Principal {
    ic_cdk::api::caller()
}

#[cfg(test)]
pub fn get_time() -> u64 {
    123456789
}

#[cfg(not(test))]
pub fn get_time() -> u64 {
    ic_cdk::api::time()
}

#[cfg(test)]
pub fn is_anonymous(princ: String) -> bool {
    false
}

#[cfg(not(test))]
pub fn is_anonymous(princ: String) -> bool {
    princ.len() < 10
}

