use crate::application_service::ApplicationServiceTrait;
use crate::http::requests::{AccountResponse};
use crate::mapper::account_mapper::account_to_account_response;
use crate::mapper::persona_mapper::{persona_request_to_persona, persona_to_persona_response};
use crate::repository::persona_repo::{PersonaRepoTrait};
use crate::requests::{PersonaRequest, PersonaResponse};
use crate::response_mapper::{HttpResponse, to_error_response, to_success_response};
use crate::util::validation_util::validate_frontend_length;

pub trait PersonaServiceTrait {
    fn create_persona(&self, persona_r: PersonaRequest) -> HttpResponse<AccountResponse>;
    fn read_personas(&self) -> HttpResponse<Vec<PersonaResponse>>;
}

#[derive(Default)]
pub struct PersonaService<T, N> {
    persona_repo: T,
    application_service: N,
}

impl<T: PersonaRepoTrait, N: ApplicationServiceTrait> PersonaServiceTrait for PersonaService<T, N> {
    fn create_persona(&self, persona_r: PersonaRequest) -> HttpResponse<AccountResponse> {
        if !validate_frontend_length(&persona_r) {
            return to_error_response("Invalid persona");
        }
        let created_persona = persona_request_to_persona(persona_r);

        if self.application_service.is_over_the_limit(&created_persona.domain) {
            return to_error_response("It's impossible to link this domain. Over limit.");
        }

        match self.persona_repo.store_persona(created_persona) {
            Some(t) => {
                to_success_response(account_to_account_response(t))
            }
            None => to_error_response("Unable to store persona.")
        }
    }

    fn read_personas(&self) -> HttpResponse<Vec<PersonaResponse>> {
        match self.persona_repo.get_personas() {
            Some(personas) => {
                let personas_r = personas.iter()
                    .map(|l| persona_to_persona_response(l.to_owned()))
                    .collect();
                to_success_response(personas_r)
            }
            None => to_error_response("Unable to find Account.")
        }
    }
}
