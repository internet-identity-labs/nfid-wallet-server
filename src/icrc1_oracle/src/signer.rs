//! Code for interacting with the chain fusion signer.
use std::sync::LazyLock;

use candid::{Nat, Principal};
use ic_cdk::api::{
    call::call_with_payment128,
};
use ic_cycles_ledger_client::{
    Account, AllowanceArgs, ApproveArgs, CyclesLedgerService, DepositArgs, DepositResult,
    ApproveError,
};
use ic_ledger_types::Subaccount;
use serde_bytes::ByteBuf;

use super::{CandidType, Deserialize, Serialize};

const SUB_ACCOUNT_ZERO: Subaccount = Subaccount([0; 32]);
pub const DEFAULT_CYCLES_LEDGER_TOP_UP_THRESHOLD: u128 = 50_000_000_000_000; // 50T
pub const DEFAULT_CYCLES_LEDGER_TOP_UP_PERCENTAGE: u8 = 50;
pub const MIN_PERCENTAGE: u8 = 1;
pub const MAX_PERCENTAGE: u8 = 99;
const MAINNET_CYCLES_LEDGER_CANISTER_ID: &str = "um5iw-rqaaa-aaaaq-qaaba-cai";
const MAINNET_SIGNER_CANISTER_ID: &str = "grghe-syaaa-aaaar-qabyq-cai";
const LEDGER_FEE: u64 = 1_000_000_000u64;
const SIGNER_FEE: u64 = 80_000_000_000;
const SIGNING_OPS_PER_LOGIN: u64 = 36;

pub static CYCLES_LEDGER: LazyLock<Principal> = LazyLock::new(|| {
    Principal::from_text(option_env!("CANISTER_ID_CYCLES_LEDGER").unwrap_or(MAINNET_CYCLES_LEDGER_CANISTER_ID))
        .unwrap_or_else(|e| unreachable!("The cycles_ledger canister ID from DFX and mainnet are valid and should have been parsed.  Is this being compiled in some strange way? {e}"))
});
pub static SIGNER: LazyLock<Principal> = LazyLock::new(|| {
    Principal::from_text(option_env!("CANISTER_ID_SIGNER").unwrap_or(MAINNET_SIGNER_CANISTER_ID))
        .unwrap_or_else(|e| unreachable!("The signer canister ID from mainnet or dfx valid and should have been parsed.  Is this being compiled in some strange way? {e}"))
});

const fn per_user_cycles_allowance() -> u64 {
    LEDGER_FEE + (LEDGER_FEE + SIGNER_FEE) * SIGNING_OPS_PER_LOGIN
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum TopUpCyclesLedgerError {
    CouldNotGetBalanceFromCyclesLedger,
    InvalidArgPercentageOutOfRange { percentage: u8, min: u8, max: u8 },
    CouldNotTopUpCyclesLedger { available: Nat, tried_to_send: Nat },
}
#[derive(CandidType, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum GetAllowedCyclesError {
    FailedToContactCyclesLedger,
    Other(String),
}
#[derive(CandidType, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum AllowSigningError {
    Other(String),
    FailedToContactCyclesLedger,
    ApproveError(ApproveError),
    PowChallenge(ChallengeCompletionError),
}
#[derive(CandidType, Deserialize, Clone, Eq, PartialEq, Debug)]
pub enum ChallengeCompletionError {
    MissingChallenge,
    InvalidNonce,
    MissingUserProfile,
    ExpiredChallenge,
    ChallengeAlreadySolved,
}
#[derive(CandidType, Deserialize, Debug, Clone, Eq, PartialEq, Default)]
pub struct TopUpCyclesLedgerRequest {
    pub threshold: Option<Nat>,
    pub percentage: Option<u8>,
}
#[derive(CandidType, Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct TopUpCyclesLedgerResponse {
    pub ledger_balance: Nat,
    pub backend_cycles: Nat,
    pub topped_up: Nat,
}
impl TopUpCyclesLedgerRequest {
    pub fn check(&self) -> Result<(), TopUpCyclesLedgerError> {
        if let Some(percentage) = self.percentage {
            if !(MIN_PERCENTAGE..=MAX_PERCENTAGE).contains(&percentage) {
                return Err(TopUpCyclesLedgerError::InvalidArgPercentageOutOfRange {
                    percentage,
                    min: MIN_PERCENTAGE,
                    max: MAX_PERCENTAGE,
                });
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn threshold(&self) -> Nat {
        self.threshold
            .clone()
            .unwrap_or(Nat::from(DEFAULT_CYCLES_LEDGER_TOP_UP_THRESHOLD))
    }

    #[must_use]
    pub fn percentage(&self) -> u8 {
        self.percentage
            .unwrap_or(DEFAULT_CYCLES_LEDGER_TOP_UP_PERCENTAGE)
    }
}

#[derive(CandidType, Serialize, Deserialize, Clone, Eq, PartialEq, Debug)]
pub enum TopUpCyclesLedgerResult {
    Ok(TopUpCyclesLedgerResponse),
    Err(TopUpCyclesLedgerError),
}
impl From<Result<TopUpCyclesLedgerResponse, TopUpCyclesLedgerError>> for TopUpCyclesLedgerResult {
    fn from(result: Result<TopUpCyclesLedgerResponse, TopUpCyclesLedgerError>) -> Self {
        match result {
            Ok(res) => TopUpCyclesLedgerResult::Ok(res),
            Err(err) => TopUpCyclesLedgerResult::Err(err),
        }
    }
}
pub async fn get_allowed_cycles() -> Result<Nat, GetAllowedCyclesError> {
    let cycles_ledger: Principal = *CYCLES_LEDGER;
    let signer: Principal = *SIGNER;
    let caller = ic_cdk::caller();
    let allowance_args = AllowanceArgs {
        account: Account {
            owner: ic_cdk::id(),
            subaccount: None,
        },
        spender: Account {
            owner: signer,
            subaccount: Some(principal2account(&caller)),
        },
    };
    let (allowance,) = CyclesLedgerService(cycles_ledger)
        .icrc_2_allowance(&allowance_args)
        .await
        .map_err(|_| GetAllowedCyclesError::FailedToContactCyclesLedger)?;

    Ok(allowance.allowance)
}

pub async fn allow_signing(allowed_cycles: Option<u64>) -> Result<Nat, AllowSigningError> {
    let cycles_ledger: Principal = *CYCLES_LEDGER;
    let signer: Principal = *SIGNER;
    let caller = ic_cdk::caller();
    let amount = Nat::from(allowed_cycles.unwrap_or_else(per_user_cycles_allowance));
    CyclesLedgerService(cycles_ledger)
        .icrc_2_approve(&ApproveArgs {
            spender: Account {
                owner: signer,
                subaccount: Some(principal2account(&caller)),
            },
            amount: amount.clone(),
            created_at_time: None,
            expected_allowance: None,
            expires_at: None,
            fee: None,
            from_subaccount: None,
            memo: None,
        })
        .await
        .map_err(|_| AllowSigningError::FailedToContactCyclesLedger)?
        .0
        .map_err(AllowSigningError::ApproveError)?;
    Ok(amount)
}


#[must_use]
pub fn principal2account(principal: &Principal) -> ByteBuf {
    let hex_str = ic_ledger_types::AccountIdentifier::new(principal, &SUB_ACCOUNT_ZERO).to_hex();
    hex::decode(&hex_str)
        .unwrap_or_else(|_| {
            unreachable!(
                "Failed to decode hex account identifier we just created: {}",
                hex_str
            )
        })
        .into()
}


pub async fn top_up_cycles_ledger(request: TopUpCyclesLedgerRequest) -> TopUpCyclesLedgerResult {
    match request.check() {
        Ok(()) => {}
        Err(err) => return TopUpCyclesLedgerResult::Err(err),
    }
    let cycles_ledger = CyclesLedgerService(*CYCLES_LEDGER);
    let account = Account {
        owner: ic_cdk::id(),
        subaccount: None,
    };
    let (ledger_balance,): (Nat,) = match cycles_ledger
        .icrc_1_balance_of(&account)
        .await
        .map_err(|_| TopUpCyclesLedgerError::CouldNotGetBalanceFromCyclesLedger)
    {
        Ok(res) => res,
        Err(err) => return TopUpCyclesLedgerResult::Err(err),
    };

    let backend_cycles = Nat::from(ic_cdk::api::canister_balance128());

    if ledger_balance < request.threshold() {
        let to_send = backend_cycles.clone() / Nat::from(100u32) * Nat::from(request.percentage());
        let to_retain = backend_cycles.clone() - to_send.clone();

        let arg = DepositArgs {
            to: account,
            memo: None,
        };
        let to_send_128: u128 =
            to_send.clone().0.try_into().unwrap_or_else(|err| {
                unreachable!("Failed to convert cycle amount to u128: {}", err)
            });
        let (result,): (DepositResult,) =
            match call_with_payment128(*CYCLES_LEDGER, "deposit", (arg,), to_send_128)
                .await
                .map_err(|_| TopUpCyclesLedgerError::CouldNotTopUpCyclesLedger {
                    available: backend_cycles,
                    tried_to_send: to_send.clone(),
                }) {
                Ok(res) => res,
                Err(err) => return TopUpCyclesLedgerResult::Err(err),
            };
        let new_ledger_balance = result.balance;

        Ok(TopUpCyclesLedgerResponse {
            ledger_balance: new_ledger_balance,
            backend_cycles: to_retain,
            topped_up: to_send,
        })
        .into()
    } else {
        Ok(TopUpCyclesLedgerResponse {
            ledger_balance,
            backend_cycles,
            topped_up: Nat::from(0u32),
        })
        .into()
    }
}
