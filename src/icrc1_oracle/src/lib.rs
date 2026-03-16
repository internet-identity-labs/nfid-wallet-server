use core::hash::Hash;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use candid::{export_service};
use candid::{CandidType, Nat, Principal};
use ic_cdk::api::call::CallResult;
use ic_cdk::{call, caller, id, storage, trap};
use ic_cdk_macros::*;
use serde::{Deserialize, Serialize};

use crate::signer::{
    btc_principal_to_p2wpkh_address, estimate_fee, get_all_utxos, get_fee_per_byte,
    utxos_selection, BtcSelectUserUtxosFeeResult, SelectedUtxosFeeError, SelectedUtxosFeeRequest,
    SelectedUtxosFeeResponse, TopUpCyclesLedgerRequest, MIN_CONFIRMATIONS_ACCEPTED_BTC_TX,
};

mod signer;
mod timer_service;

#[derive(CandidType, Deserialize, Clone, Debug, Hash, PartialEq, Eq, Serialize)]
pub enum Category {
    Spam,
    Known,
    Native,
    ChainFusion,
    ChainFusionTestnet,
    Community,
    Sns,
}

#[derive(CandidType, Debug, Clone, Deserialize)]
pub struct CanisterIdRequest {
    #[serde(rename = "canister_id")]
    pub canister_id: Principal,
}

#[derive(Serialize, Deserialize, CandidType, Clone, PartialEq, Eq, Debug)]
pub enum CanisterStatus {
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "stopping")]
    Stopping,
    #[serde(rename = "stopped")]
    Stopped,
}

#[derive(Deserialize, CandidType, Clone, PartialEq, Eq, Debug)]
pub struct DefiniteCanisterSettings {
    controllers: Vec<Principal>,
    compute_allocation: Nat,
    memory_allocation: Nat,
    freezing_threshold: Nat,
}

#[derive(Deserialize, CandidType, Clone, PartialEq, Eq, Debug)]
pub struct CanisterStatusResponse {
    status: CanisterStatus,
    settings: DefiniteCanisterSettings,
    module_hash: Option<Vec<u8>>,
    memory_size: Nat,
    cycles: Nat,
    freezing_threshold: Nat,
}

#[derive(CandidType, Deserialize, Clone, Debug, Hash, Serialize, PartialEq)]
pub struct Conf {
    pub im_canister: Option<Principal>,
    pub operator: Option<Principal>,
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug, Eq)]
pub struct ICRC1 {
    pub index: Option<String>,
    pub ledger: String,
    pub name: String,
    pub logo: Option<String>,
    pub symbol: String,
    pub category: Category,
    pub decimals: u8,
    pub fee: Nat,
    pub root_canister_id: Option<String>,
    pub date_added: u64,
}

impl Hash for ICRC1 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ledger.hash(state);
    }
}

impl PartialEq for ICRC1 {
    fn eq(&self, other: &Self) -> bool {
        self.ledger == other.ledger
    }
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug, Eq)]
pub struct NeuronData {
    pub name: String,
    pub ledger: String,
    pub neuron_id: String,
    pub date_added: u64,
}

impl Hash for NeuronData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ledger.hash(state);
    }
}

impl PartialEq for NeuronData {
    fn eq(&self, other: &Self) -> bool {
        self.ledger == other.ledger
    }
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug, Hash, PartialEq, Eq)]
pub struct ICRC1Request {
    pub index: Option<String>,
    pub ledger: String,
    pub name: String,
    pub logo: Option<String>,
    pub symbol: String,
    pub decimals: u8,
    pub fee: Nat,
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug, Hash, PartialEq, Eq)]
pub enum DiscoveryStatus {
    New,
    Updated,
    Verified,
    Spam,
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug)]
pub struct DiscoveryApp {
    pub id: u32,
    pub derivation_origin: Option<String>,
    pub hostname: String,
    pub url: Option<String>,
    pub name: Option<String>,
    pub icon: Option<String>,
    pub desc: Option<String>,
    pub is_global: bool,
    pub is_anonymous: bool,
    pub unique_users: u64,
    pub status: DiscoveryStatus,
}

impl Hash for DiscoveryApp {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for DiscoveryApp {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for DiscoveryApp {}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug, Hash, PartialEq, Eq)]
pub enum LoginType {
    Global,
    Anonymous,
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug)]
pub struct DiscoveryVisitRequest {
    pub derivation_origin: Option<String>,
    pub hostname: String,
    pub login: LoginType,
}

thread_local! {
     static CONFIG: RefCell<Conf> = const { RefCell::new( Conf {
        im_canister: None,
        operator: None
    }) };
    pub static ICRC_REGISTRY: RefCell<HashSet<ICRC1>> = RefCell::new(HashSet::default());
    pub static NEURON_REGISTRY: RefCell<HashSet<NeuronData>> = RefCell::new(HashSet::default());
    pub static DISCOVERY_REGISTRY: RefCell<HashSet<DiscoveryApp>> = RefCell::new(HashSet::default());
    // Key: app id, Value: set of root_id strings (unique visitors per app)
    pub static DISCOVERY_VISITORS: RefCell<HashMap<u32, HashSet<String>>> = RefCell::new(HashMap::new());
}

/// Persists a single ICRC1 canister's metadata into the canister's storage.
#[update]
pub async fn store_icrc1_canister(request: ICRC1Request) {
    get_root_id().await;
    Principal::from_text(request.ledger.clone()).unwrap_or_else(|_| {
        trap("Invalid ledger principal");
    });
    if request.index.is_some() {
        Principal::from_text(
            request
                .index
                .clone()
                .expect("The request.index failed after existence check."),
        )
        .unwrap_or_else(|_| {
            trap("Invalid index principal");
        });
    }
    ICRC_REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        let canister_id = ICRC1 {
            index: request.index,
            ledger: request.ledger,
            name: request.name,
            logo: request.logo,
            symbol: request.symbol,
            category: Category::Community,
            decimals: request.decimals,
            fee: request.fee,
            root_canister_id: None,
            date_added: ic_cdk::api::time(),
        };
        registry.replace(canister_id);
    });
}

/// Invoked when the canister starts.
/// Initializes the application with `Conf` parameters and saves them to storage.
#[init]
pub async fn init(conf: Option<Conf>) {
    if let Some(conf) = conf {
        CONFIG.with(|storage| {
            storage.replace(conf);
        });
    }
}

/// Returns all persisted ICRC1 canisters.
#[query]
pub async fn get_all_icrc1_canisters() -> HashSet<ICRC1> {
    ICRC_REGISTRY.with(|registry| {
        let registry = registry.borrow();
        registry.clone()
    })
}

/// Returns amount of all persisted ICRC1 canisters.
#[query]
pub async fn count_icrc1_canisters() -> u64 {
    ICRC_REGISTRY.with(|registry| {
        let registry = registry.borrow();
        registry.len() as u64
    })
}

/// Retirns paginated response
#[query]
pub async fn get_icrc1_paginated(offset: u64, limit: u64) -> Vec<ICRC1> {
    ICRC_REGISTRY.with(|registry| {
        let registry = registry.borrow();
        registry
            .iter()
            .skip(offset as usize)
            .take(limit as usize)
            .cloned()
            .collect()
    })
}

/// Replaces the existing ICRC1 canisters with the provided list.
#[update]
pub async fn replace_icrc1_canisters(icrc1: Vec<ICRC1>) {
    trap_if_not_authenticated_admin();
    ICRC_REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        for canister in icrc1 {
            registry.replace(canister);
        }
    })
}

/// Persists an array of ICRC1 canisters under a specified category.
#[update]
pub async fn store_new_icrc1_canisters(icrc1: Vec<ICRC1>) {
    trap_if_not_authenticated_admin();
    ICRC_REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        for mut canister in icrc1 {
            let existent_canister = registry.get(&canister);
            //if canister exists - update metadata. Sometimes SNS logo can be updated silently
            if let Some(existent) = existent_canister {
                canister.index = existent.index.clone();
                canister.date_added = existent.date_added;
                registry.replace(canister);
            } else {
                registry.insert(canister);
            }
        }
    })
}

/// Removes an ICRC1 canister by its ledger principal.
#[update]
pub async fn remove_icrc1_canister(ledger: String) {
    trap_if_not_authenticated_admin();
    ICRC_REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        registry.retain(|canister| canister.ledger != ledger);
    });
}

/// Tracks a visit to a dapp by hostname. Updates unique_users, is_global, is_anonymous.
#[update]
pub async fn store_discovery_app(request: DiscoveryVisitRequest) {
    let root_id = get_root_id().await;

    let found_app = DISCOVERY_REGISTRY.with(|registry| {
        registry
            .borrow()
            .iter()
            .find(|app| app.hostname == request.hostname)
            .cloned()
    });

    let Some(mut app) = found_app else { return };

    let is_new_visitor = DISCOVERY_VISITORS.with(|visitors| {
        visitors
            .borrow_mut()
            .entry(app.id)
            .or_insert_with(HashSet::new)
            .insert(root_id)
    });

    if is_new_visitor {
        app.unique_users += 1;
    }

    match request.login {
        LoginType::Global if !app.is_global => app.is_global = true,
        LoginType::Anonymous if !app.is_anonymous => app.is_anonymous = true,
        _ => {}
    }

    DISCOVERY_REGISTRY.with(|registry| {
        registry.borrow_mut().replace(app);
    });
}

/// Returns true if calling store_discovery_app would result in any state change:
/// caller is a new unique visitor, or the login type would flip is_global / is_anonymous.
/// Uses caller() directly (no inter-canister call) — suitable for a query.
#[query]
pub fn is_unique(request: DiscoveryVisitRequest) -> bool {
    let visitor_id = caller().to_text();

    let found_app = DISCOVERY_REGISTRY.with(|registry| {
        registry
            .borrow()
            .iter()
            .find(|app| app.hostname == request.hostname)
            .cloned()
    });

    let Some(app) = found_app else { return false };

    let is_new_visitor = DISCOVERY_VISITORS.with(|visitors| {
        !visitors
            .borrow()
            .get(&app.id)
            .map(|s| s.contains(&visitor_id))
            .unwrap_or(false)
    });

    if is_new_visitor {
        return true;
    }

    match request.login {
        LoginType::Global if !app.is_global => true,
        LoginType::Anonymous if !app.is_anonymous => true,
        _ => false,
    }
}

/// Returns a paginated list of DiscoveryApps.
#[query]
pub fn get_discovery_app_paginated(offset: u64, limit: u64) -> Vec<DiscoveryApp> {
    DISCOVERY_REGISTRY.with(|registry| {
        registry
            .borrow()
            .iter()
            .skip(offset as usize)
            .take(limit as usize)
            .cloned()
            .collect()
    })
}

/// Upserts a batch of DiscoveryApps by id (admin-facing). Does not clear existing entries.
#[update]
pub async fn replace_all_discovery_app(apps: Vec<DiscoveryApp>) {
    trap_if_not_authenticated_admin();
    DISCOVERY_REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        for app in apps {
            registry.replace(app);
        }
    });
}

/// Clears all DiscoveryApps from the registry (admin-facing).
#[update]
pub async fn clear_discovery_apps() {
    trap_if_not_authenticated_admin();
    DISCOVERY_REGISTRY.with(|registry| {
        registry.borrow_mut().clear();
    });
}

/// Sets the operator principal.
#[update]
async fn set_operator(operator: Principal) {
    let controllers = get_controllers().await;
    if !controllers.contains(&ic_cdk::caller()) {
        trap("Unauthorized: caller is not a controller");
    }
    CONFIG.with(|config| {
        let mut config = config.borrow_mut();
        config.operator = Some(operator);
    });
}

#[query]
async fn get_all_neurons() -> Vec<NeuronData> {
    NEURON_REGISTRY.with(|registry| {
        let registry = registry.borrow();
        registry.clone().into_iter().collect()
    })
}

#[update]
async fn up_cycles(amount: Option<u128>) {
    trap_if_not_authenticated_admin();
    let _ = signer::top_up_cycles_ledger(TopUpCyclesLedgerRequest {
        threshold: Some(Nat::from(amount.unwrap_or(2_000_000_000_000u128))),
        percentage: None,
    })
    .await;
}

#[update]
async fn replace_all_neurons(neurons: Vec<NeuronData>) {
    trap_if_not_authenticated_admin();
    NEURON_REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        registry.clear();
        for neuron in neurons {
            registry.insert(neuron);
        }
    })
}

#[update]
pub async fn allow_signing() {
    let _ = signer::allow_signing(None).await;
}

#[update]
pub async fn btc_select_user_utxos_fee(
    params: SelectedUtxosFeeRequest,
) -> BtcSelectUserUtxosFeeResult {
    async fn inner(
        params: SelectedUtxosFeeRequest,
    ) -> Result<SelectedUtxosFeeResponse, SelectedUtxosFeeError> {
        let principal = ic_cdk::caller();
        let source_address = btc_principal_to_p2wpkh_address(params.network, &principal)
            .await
            .map_err(|msg| SelectedUtxosFeeError::InternalError { msg })?;
        let all_utxos = get_all_utxos(
            params.network,
            source_address.clone(),
            Some(
                params
                    .min_confirmations
                    .unwrap_or(MIN_CONFIRMATIONS_ACCEPTED_BTC_TX),
            ),
        )
        .await
        .map_err(|msg| SelectedUtxosFeeError::InternalError { msg })?;

        let median_fee_millisatoshi_per_vbyte = get_fee_per_byte(params.network)
            .await
            .map_err(|msg| SelectedUtxosFeeError::InternalError { msg })?;
        // We support sending to one destination only.
        // Therefore, the outputs are the destination and the source address for the change.
        let output_count = 2;
        let mut available_utxos = all_utxos.clone();
        let selected_utxos =
            utxos_selection(params.amount_satoshis, &mut available_utxos, output_count);

        // Fee calculation might still take into account default tx size and expected output.
        // But if there are no selcted utxos, no tx is possible. Therefore, no fee should be
        // present.
        if selected_utxos.is_empty() {
            return Ok(SelectedUtxosFeeResponse {
                utxos: selected_utxos,
                fee_satoshis: 0,
            });
        }

        let fee_satoshis = estimate_fee(
            selected_utxos.len() as u64,
            median_fee_millisatoshi_per_vbyte,
            output_count as u64,
        );

        Ok(SelectedUtxosFeeResponse {
            utxos: selected_utxos,
            fee_satoshis,
        })
    }
    inner(params).await.into()
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug, Eq)]
pub struct ICRC1Memory {
    pub index: Option<String>,
    pub ledger: String,
    pub name: String,
    pub logo: Option<String>,
    pub symbol: String,
    pub category: Category,
    pub decimals: u8,
    pub fee: Nat,
    pub root_canister_id: Option<String>,
    pub date_added: Option<u64>,
}

impl Hash for ICRC1Memory {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ledger.hash(state);
    }
}

impl PartialEq for ICRC1Memory {
    fn eq(&self, other: &Self) -> bool {
        self.ledger == other.ledger
    }
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
struct Memory {
    registry: HashSet<ICRC1Memory>,
    neurons: Option<HashSet<NeuronData>>,
    config: Conf,
    discovery_apps: Option<HashSet<DiscoveryApp>>,
    discovery_visitors: Option<HashMap<u32, HashSet<String>>>,
}

/// Applies changes prior to the canister upgrade.
#[pre_upgrade]
pub fn stable_save() {
    let registry = ICRC_REGISTRY.with(|registry| {
        let registry1 = registry.borrow();
        registry1
            .iter()
            .map(|x| ICRC1Memory {
                index: x.index.clone(),
                ledger: x.ledger.clone(),
                name: x.name.clone(),
                logo: x.logo.clone(),
                symbol: x.symbol.clone(),
                category: x.category.clone(),
                decimals: x.decimals,
                fee: x.fee.clone(),
                root_canister_id: x.root_canister_id.clone(),
                date_added: Some(x.date_added),
            })
            .collect()
    });
    let config = CONFIG.with(|config| {
        let config = config.borrow();
        config.clone()
    });
    let neurons = NEURON_REGISTRY.with(|registry| {
        let registry = registry.borrow();
        registry.clone()
    });
    let discovery_apps = DISCOVERY_REGISTRY.with(|registry| registry.borrow().clone());
    let discovery_visitors = DISCOVERY_VISITORS.with(|v| v.borrow().clone());
    let mem = Memory {
        registry,
        config,
        neurons: Some(neurons),
        discovery_apps: Some(discovery_apps),
        discovery_visitors: Some(discovery_visitors),
    };
    storage::stable_save((mem,))
        .expect("Stable save exited unexpectedly: unable to save data to stable memory.");
}

/// Applies changes following the canister upgrade.
#[post_upgrade]
pub fn stable_restore() {
    let (mo,): (Memory,) = storage::stable_restore()
        .expect("Stable restore exited unexpectedly: unable to restore data from stable memory.");
    CONFIG.with(|config| {
        let mut config = config.borrow_mut();
        *config = mo.config.clone();
    });
    timer_service::start_timer(3600);
    ICRC_REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        *registry = mo
            .registry
            .iter()
            .map(|x| ICRC1 {
                index: x.index.clone(),
                ledger: x.ledger.clone(),
                name: x.name.clone(),
                logo: x.logo.clone(),
                symbol: x.symbol.clone(),
                category: x.category.clone(),
                decimals: x.decimals,
                fee: x.fee.clone(),
                root_canister_id: x.root_canister_id.clone(),
                date_added: x.date_added.unwrap_or(ic_cdk::api::time()),
            })
            .collect()
    });
    NEURON_REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        *registry = mo.neurons.unwrap_or_default().clone();
    });
    DISCOVERY_REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        *registry = mo.discovery_apps.unwrap_or_default();
    });
    DISCOVERY_VISITORS.with(|visitors| {
        *visitors.borrow_mut() = mo.discovery_visitors.unwrap_or_default();
    });
}

#[test]
fn sub_account_test() {}
export_service!();

#[ic_cdk_macros::query(name = "__get_candid_interface")]
fn export_candid() -> String {
    __export_service()
}

async fn get_root_id() -> String {
    match CONFIG.with(|c| c.borrow_mut().im_canister) {
        None => caller().to_text(), // Return caller for testing purposes when im_canister is None
        Some(canister) => {
            let princ = caller();
            match call(canister, "get_root_by_principal", (princ.to_text(), 0)).await {
                Ok((Some(root_id),)) => root_id,
                Ok((None,)) => trap("No root found for this principal"),
                Err((_, err)) => trap(&format!("Failed to request IM: {}", err)),
            }
        }
    }
}

fn trap_if_not_authenticated_admin() {
    let princ = caller();
    match CONFIG.with(|c| c.borrow_mut().operator) {
        None => trap("Unauthorised"),
        Some(operator) => {
            if !operator.eq(&princ) {
                trap("Unauthorised")
            }
        }
    }
}

async fn get_controllers() -> Vec<Principal> {
    let res: CallResult<(ic_cdk::api::management_canister::main::CanisterStatusResponse,)> = call(
        Principal::management_canister(),
        "canister_status",
        (CanisterIdRequest { canister_id: id() },),
    )
    .await;
    res
        .expect("Get controllers function exited unexpectedly: inter-canister call to management canister for canister_status returned an empty result.")
        .0.settings.controllers
}
