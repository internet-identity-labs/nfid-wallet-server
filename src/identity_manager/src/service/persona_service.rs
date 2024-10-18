use crate::application_service::ApplicationServiceTrait;
use crate::mapper::persona_mapper::persona_to_persona_response;
use crate::repository::persona_repo::PersonaRepoTrait;
use crate::requests::PersonaResponse;
use crate::response_mapper::{to_error_response, to_success_response, HttpResponse};

#[deprecated()]
pub trait PersonaServiceTrait {
    fn read_personas(&self) -> HttpResponse<Vec<PersonaResponse>>;
}

#[derive(Default)]
pub struct PersonaService<T, N> {
    pub persona_repo: T,
    pub application_service: N,
}

impl<T: PersonaRepoTrait, N: ApplicationServiceTrait> PersonaServiceTrait for PersonaService<T, N> {
    fn read_personas(&self) -> HttpResponse<Vec<PersonaResponse>> {
        match self.persona_repo.get_personas() {
            Some(personas) => {
                let personas_r = personas
                    .iter()
                    .map(|l| persona_to_persona_response(l.to_owned()))
                    .collect();
                to_success_response(personas_r)
            }
            None => to_error_response("Unable to find Account."),
        }
    }
}
