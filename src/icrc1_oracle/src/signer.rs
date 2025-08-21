//! Code for interacting with the chain fusion signer.
use std::sync::LazyLock;

use candid::{Nat, Principal};
use ic_cdk::api::management_canister::bitcoin::{
    bitcoin_get_current_fee_percentiles, bitcoin_get_utxos, BitcoinNetwork,
    GetCurrentFeePercentilesRequest, GetUtxosRequest, GetUtxosResponse, MillisatoshiPerByte, Utxo,
    UtxoFilter,
};
use ic_cdk::api::{
    call::call_with_payment128,
    management_canister::ecdsa::{
        ecdsa_public_key, EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgument,
    },
    time,
};
use ic_cycles_ledger_client::{
    Account, AllowanceArgs, ApproveArgs, ApproveError, CyclesLedgerService, DepositArgs,
    DepositResult,
};
use serde::{Deserialize, Serialize};

use bitcoin::{Address, CompressedPublicKey, Network};
use ic_ledger_types::Subaccount;
use serde_bytes::ByteBuf;

use super::CandidType;

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
const INPUT_SIZE_VBYTES: u64 = 68;
const OUTPUT_SIZE_VBYTES: u64 = 31;
const TX_OVERHEAD_VBYTES: u64 = 11;
pub const MIN_CONFIRMATIONS_ACCEPTED_BTC_TX: u32 = 6;
pub const MAX_UTXOS_LEN: usize = 128;
pub const MAX_TXID_BYTES: usize = 32;

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

/// Identifier of [Utxo].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct Outpoint {
    /// Transaction Identifier.
    pub txid: Vec<u8>,
    /// A implicit index number.
    pub vout: u32,
}

#[derive(CandidType, Deserialize, Clone, Eq, PartialEq, Debug)]
pub enum SelectedUtxosFeeError {
    InternalError { msg: String },
    PendingTransactions,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Eq, PartialEq, Debug)]
pub struct SelectedUtxosFeeResponse {
    pub utxos: Vec<Utxo>,
    pub fee_satoshis: u64,
}

#[derive(CandidType, Deserialize, Clone, Eq, PartialEq, Debug)]
pub enum BtcSelectUserUtxosFeeResult {
    /// The fee was selected successfully.
    Ok(SelectedUtxosFeeResponse),
    /// The fee was not selected due to an error.
    Err(SelectedUtxosFeeError),
}

impl From<Result<SelectedUtxosFeeResponse, SelectedUtxosFeeError>> for BtcSelectUserUtxosFeeResult {
    fn from(result: Result<SelectedUtxosFeeResponse, SelectedUtxosFeeError>) -> Self {
        match result {
            Ok(response) => BtcSelectUserUtxosFeeResult::Ok(response),
            Err(err) => BtcSelectUserUtxosFeeResult::Err(err),
        }
    }
}

#[derive(CandidType, Deserialize, Clone, Eq, PartialEq, Debug)]
pub struct SelectedUtxosFeeRequest {
    pub amount_satoshis: u64,
    pub network: BitcoinNetwork,
    pub min_confirmations: Option<u32>,
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
        self.threshold.clone().unwrap_or(Nat::from(DEFAULT_CYCLES_LEDGER_TOP_UP_THRESHOLD))
    }

    #[must_use]
    pub fn percentage(&self) -> u8 {
        self.percentage.unwrap_or(DEFAULT_CYCLES_LEDGER_TOP_UP_PERCENTAGE)
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
        account: Account { owner: ic_cdk::id(), subaccount: None },
        spender: Account { owner: signer, subaccount: Some(principal2account(&caller)) },
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
            spender: Account { owner: signer, subaccount: Some(principal2account(&caller)) },
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
            unreachable!("Failed to decode hex account identifier we just created: {}", hex_str)
        })
        .into()
}

pub async fn top_up_cycles_ledger(request: TopUpCyclesLedgerRequest) -> TopUpCyclesLedgerResult {
    match request.check() {
        Ok(()) => {}
        Err(err) => return TopUpCyclesLedgerResult::Err(err),
    }
    let cycles_ledger = CyclesLedgerService(*CYCLES_LEDGER);
    let account = Account { owner: ic_cdk::id(), subaccount: None };
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

        let arg = DepositArgs { to: account, memo: None };
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
        Ok(TopUpCyclesLedgerResponse { ledger_balance, backend_cycles, topped_up: Nat::from(0u32) })
            .into()
    }
}

pub trait Validate {
    /// Verifies that an object is semantically valid.
    ///
    /// # Errors
    /// - If the object is invalid.
    fn validate(&self) -> Result<(), candid::Error>;
    /// Returns the object if it is semantically valid.
    ///
    /// # Errors
    /// - If the object is invalid.
    fn validated(self) -> Result<Self, candid::Error>
    where
        Self: Sized,
    {
        self.validate().map(|()| self)
    }
}

impl Validate for SelectedUtxosFeeResponse {
    fn validate(&self) -> Result<(), candid::Error> {
        validate_utxo_vec(&self.utxos)
    }
}

fn validate_utxo(utxo: &Utxo) -> Result<(), candid::Error> {
    let len = utxo.outpoint.txid.len();
    if len > MAX_TXID_BYTES {
        return Err(candid::Error::msg(format!(
            "Transaction ID in utxo has too many bytes: {len} > {MAX_TXID_BYTES}"
        )));
    }
    Ok(())
}

fn validate_utxo_vec(utxos: &[Utxo]) -> Result<(), candid::Error> {
    if utxos.len() > MAX_UTXOS_LEN {
        return Err(candid::Error::msg(format!(
            "Too many UTXOs: {} > {}",
            utxos.len(),
            MAX_UTXOS_LEN
        )));
    }
    for utxo in utxos {
        validate_utxo(utxo)?;
    }
    Ok(())
}

async fn cfs_ecdsa_pubkey_of(principal: &Principal) -> Result<Vec<u8>, String> {
    let cfs_canister_id = Principal::from_text(MAINNET_SIGNER_CANISTER_ID).unwrap();
    let ecdsa_key_name = "key_1";
    let btc_schema = vec![0_u8];
    let derivation_path = vec![btc_schema, principal.as_slice().to_vec()];
    if let Ok((key,)) = ecdsa_public_key(EcdsaPublicKeyArgument {
        canister_id: Some(cfs_canister_id),
        derivation_path,
        key_id: EcdsaKeyId { curve: EcdsaCurve::Secp256k1, name: ecdsa_key_name.to_string() },
    })
    .await
    {
        Ok(key.public_key)
    } else {
        Err("Failed to get ecdsa public key".to_string())
    }
}

fn transform_network(network: BitcoinNetwork) -> Network {
    match network {
        BitcoinNetwork::Mainnet => Network::Bitcoin,
        BitcoinNetwork::Testnet => Network::Testnet,
        BitcoinNetwork::Regtest => Network::Regtest,
    }
}

pub async fn btc_principal_to_p2wpkh_address(
    network: BitcoinNetwork,
    principal: &Principal,
) -> Result<String, String> {
    let ecdsa_pubkey = cfs_ecdsa_pubkey_of(principal).await?;
    if let Ok(compressed_public_key) = CompressedPublicKey::from_slice(&ecdsa_pubkey) {
        Ok(Address::p2wpkh(&compressed_public_key, transform_network(network)).to_string())
    } else {
        Err("Error getting P2WPKH from public key".to_string())
    }
}

async fn get_utxos(
    network: BitcoinNetwork,
    address: String,
    filter: Option<UtxoFilter>,
) -> Result<GetUtxosResponse, String> {
    let utxos_res = bitcoin_get_utxos(GetUtxosRequest { address, network, filter })
        .await
        .map_err(|err| err.1)?;

    Ok(utxos_res.0)
}
pub async fn get_all_utxos(
    network: BitcoinNetwork,
    address: String,
    min_confirmations: Option<u32>,
) -> Result<Vec<Utxo>, String> {
    let final_min_confirmations = if network == BitcoinNetwork::Regtest {
        // Tests with Regtest fail if min_confirmations is higher than 1.
        Some(1)
    } else {
        min_confirmations
    };
    let filter = final_min_confirmations.map(UtxoFilter::MinConfirmations);
    let mut utxos_response = get_utxos(network, address.clone(), filter).await?;

    let mut all_utxos: Vec<Utxo> = utxos_response.utxos;
    let mut next_page: Option<Vec<u8>> = utxos_response.next_page;
    while next_page.is_some() {
        utxos_response =
            get_utxos(network, address.clone(), next_page.map(UtxoFilter::Page)).await?;
        all_utxos.extend(utxos_response.utxos);
        next_page = utxos_response.next_page;
    }

    Ok(all_utxos)
}

pub async fn get_fee_per_byte(network: BitcoinNetwork) -> Result<u64, String> {
    // Get fee percentiles from previous transactions to estimate our own fee.
    let fee_percentiles = get_current_fee_percentiles(network).await?;

    if fee_percentiles.is_empty() {
        // There are no fee percentiles. This case can only happen on a regtest
        // network where there are no non-coinbase transactions. In this case,
        // we use a default of 2000 millisatoshis/byte (i.e. 2 satoshi/byte)
        Ok(2000)
    } else {
        let middle = fee_percentiles.len() / 2;
        Ok(fee_percentiles[middle])
    }
}

async fn get_current_fee_percentiles(
    network: BitcoinNetwork,
) -> Result<Vec<MillisatoshiPerByte>, String> {
    let res = bitcoin_get_current_fee_percentiles(GetCurrentFeePercentilesRequest { network })
        .await
        .map_err(|err| err.1)?;

    Ok(res.0)
}

fn tx_vsize_estimate(input_count: u64, output_count: u64) -> u64 {
    input_count * INPUT_SIZE_VBYTES + output_count * OUTPUT_SIZE_VBYTES + TX_OVERHEAD_VBYTES
}

pub fn estimate_fee(
    selected_utxos_count: u64,
    median_fee_millisatoshi_per_vbyte: u64,
    output_count: u64,
) -> u64 {
    tx_vsize_estimate(selected_utxos_count, output_count) * median_fee_millisatoshi_per_vbyte / 1000
}
const UTXOS_COUNT_THRESHOLD: usize = 1_000;
fn greedy(target: u64, available_utxos: &mut Vec<Utxo>) -> Vec<Utxo> {
    let mut solution = vec![];
    let mut goal = target;
    while goal > 0 {
        let utxo = match available_utxos.iter().max_by_key(|u| u.value) {
            Some(max_utxo) if max_utxo.value < goal => max_utxo.clone(),
            Some(_) => available_utxos
                .iter()
                .filter(|u| u.value >= goal)
                .min_by_key(|u| u.value)
                .cloned()
                .expect("bug: there must be at least one UTXO matching the criteria"),
            None => {
                // Not enough available UTXOs to satisfy the request.
                for u in solution {
                    available_utxos.push(u);
                }
                return vec![];
            }
        };
        goal = goal.saturating_sub(utxo.value);
        available_utxos.retain(|x| *x != utxo);
        solution.push(utxo);
    }

    debug_assert!(solution.is_empty() || solution.iter().map(|u| u.value).sum::<u64>() >= target);

    solution
}

pub fn utxos_selection(
    target: u64,
    available_utxos: &mut Vec<Utxo>,
    output_count: usize,
) -> Vec<Utxo> {
    let mut input_utxos = greedy(target, available_utxos);

    if input_utxos.is_empty() {
        return vec![];
    }

    if available_utxos.len() > UTXOS_COUNT_THRESHOLD {
        while input_utxos.len() < output_count + 1 {
            if let Some(min_utxo) = available_utxos.iter().min_by_key(|u| u.value) {
                let min_utxo = min_utxo.clone();
                input_utxos.push(min_utxo.clone());
                available_utxos.retain(|x| *x != min_utxo);
            } else {
                break;
            }
        }
    }

    input_utxos
}
