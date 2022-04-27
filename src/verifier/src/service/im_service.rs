use ic_cdk::{call, trap};
use ic_cdk::export::Principal;

use crate::{ConfigurationRepo, HttpResponse};

pub async fn verify_phone_number_existence(principal: String, domain: String) -> Option<String> {
    let im_canister_id = ConfigurationRepo::get().identity_manager_canister_id.clone();
    let im_response: HttpResponse<String> = match call(Principal::from_text(im_canister_id).unwrap(),
                                                       "certify_phone_number_sha2", (principal.clone(), domain.clone())).await
    {
        Ok((res, )) => res,
        Err((_, err)) => trap(&format!("Failed to request im canuster: {}", err)),
    };
    match im_response.error {
        None => {}
        Some(error) => { trap(&format!("Response error: {}", error)) }
    }
    im_response.data
}