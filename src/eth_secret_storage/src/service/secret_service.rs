use std::str::FromStr;
use ethers_core::types::Signature;
use ic_cdk::{trap, export::Principal, call};
use crate::{repository::secret_repo, constant::constant::{ErrorMessage::NoSuchApp, Message::Message}};
use super::response_service::{get_error, get_success, HttpResponse};

pub async fn get_secret_by_signature(app: String, signature: String) -> HttpResponse<String> {
    let signature: Signature = match Signature::from_str(signature.as_str()) {
        Ok(signature) => signature,
        Err(error) => return get_error(error.to_string(), 400)
    };

    let address = match signature.recover(Message.value()) {
        Ok(address) => format!("{:?}", address),
        Err(error) => return get_error(error.to_string(), 400)
    };

    let secret = secret_repo::get(app.clone(), address.clone());

    let secret = match secret {
        None => return get_error(NoSuchApp.value(), 400),
        Some(secret) => match secret {
            Some(secret) => secret,
            None => {
                let secret = generate_secret().await;
                secret_repo::save(app.clone(), address.clone(), secret.clone());
                secret
            }
        }
    };

    get_success(secret)
}

async fn generate_secret() -> String {
    let token: Vec<u8> = match call(Principal::management_canister(), "raw_rand", ()).await {
        Ok((res, )) => res,
        Err((_, err)) => trap(&format!("Failed to get salt: {}", err)),
    };

    let token: String = hex::encode(token);
    token
}