use crate::AccountRepo;
use crate::repository::account_repo::{Account, AccountRepoTrait};
use crate::repository::repo::BasicEntity;
use ic_cdk::export::candid::{CandidType, Deserialize};
#[cfg(test)]
use mockers_derive::mocked;


#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Persona {
    pub domain: String,
    pub persona_id: String,
    pub persona_name: Option<String>,
    pub base_fields: BasicEntity,
    pub domain_certified: Option<u64>,
}

#[cfg_attr(test, mocked)]
pub trait PersonaRepoTrait {
    fn get_personas(&self) -> Option<Vec<Persona>>;
    fn store_persona(&self, persona: Persona) -> Option<Account>;
}

#[derive(Default)]
pub struct PersonaRepo {
    pub account_repo: AccountRepo,
}

impl PersonaRepoTrait for PersonaRepo {
    fn get_personas(&self) -> Option<Vec<Persona>> {
        self.account_repo.get_account()
            .map(|x| x.personas.clone()) //todo &
    }

    fn store_persona(&self, persona: Persona) -> Option<Account> {
        let acc = self.account_repo.get_account();
        if acc.is_none() { return None; }
        let mut account = acc.unwrap().clone();
        account.personas.push(persona);
        self.account_repo.store_account(account)
    }
}