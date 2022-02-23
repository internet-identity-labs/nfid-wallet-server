use crate::{CredentialVariant, HttpResponse, ic_service};
use crate::CredentialVariant::PhoneNumber;
use crate::http::response_mapper::{DataResponse, ErrorResponse};
use crate::repository::account_repo::AccountRepoTrait;
use crate::repository::encrypt::account_encrypt::decrypt;
use crate::requests::PhoneNumberCredential;

pub trait CredentialServiceTrait {
    fn credentials(&self) -> HttpResponse<Vec<CredentialVariant>>;
}

#[derive(Default)]
pub struct CredentialService<T> {
    pub(crate) account_repo: T
}

impl<T: AccountRepoTrait> CredentialServiceTrait for CredentialService<T> {

    fn credentials(&self) -> HttpResponse<Vec<CredentialVariant>> {
        let principal_id = ic_service::get_caller().to_text();
        if ic_service::is_anonymous(principal_id) {
            return HttpResponse::error(403, "Anonymous user is forbidden.");
        }

        let account = self.account_repo.get_account();
        if account.is_none() {
            return HttpResponse::error(404, "Account not found.");
        }

        let credentials: Vec<CredentialVariant> = account
            .map(|x| x.phone_number)
            .flatten()
            .map(|x| decrypt(x))
            .map(|phone_number| PhoneNumberCredential {phone_number})
            .map(|x| PhoneNumber(x))
            .map(|x| vec!(x) )
            .unwrap_or_default();

        HttpResponse::data(200, credentials)
    }

}




