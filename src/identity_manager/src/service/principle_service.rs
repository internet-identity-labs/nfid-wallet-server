use crate::repository::repo::PrincipalIndex;

pub fn get_principal(persona_id: &str) -> &str {
    // let princ = &ic_cdk::api::caller().to_text().as_str(); ///todo
    match PrincipalIndex::get_principal(persona_id) {
        Some(principal_id) => {
            principal_id
        }
        None => {
            persona_id
        }
    }
}




