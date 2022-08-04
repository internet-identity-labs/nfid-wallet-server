use crate::AccountRepo;
use crate::repository::account_repo::{Account, AccountRepoTrait};
use crate::repository::repo::BasicEntity;
use ic_cdk::export::candid::{CandidType, Deserialize};
#[cfg(test)]
use mockers_derive::mocked;
use serde::{Serialize};


#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
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
    fn store_personas(&self, persona: Vec<Persona>) -> Option<Account>;
}

impl PartialEq for Persona {
    fn eq(&self, other: &Self) -> bool {
        self.persona_id == other.persona_id
        && self.domain == other.domain
    }
}
#[derive(Default)]
pub struct PersonaRepo {
    pub account_repo: AccountRepo,
}

impl PersonaRepoTrait for PersonaRepo {
    fn get_personas(&self) -> Option<Vec<Persona>> {
        self.account_repo.get_account()
            .map(|x| x.personas)
    }

    fn store_persona(&self, persona: Persona) -> Option<Account> {
        let acc = self.account_repo.get_account();
        if acc.is_none() { return None; }
        let mut account = acc.unwrap().clone();
        account.personas.push(persona);
        self.account_repo.store_account(account)
    }

    fn store_personas(&self, personas: Vec<Persona>) -> Option<Account> {
        let acc = self.account_repo.get_account();
        if acc.is_none() { return None; }
        let mut account = acc.unwrap().clone();
        account.personas = personas;
        self.account_repo.store_account(account)
    }
}