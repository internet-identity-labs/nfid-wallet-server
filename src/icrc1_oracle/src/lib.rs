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
    pub image: Option<String>,
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
    pub anonymous_principal: Option<Principal>,
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug)]
pub struct UserDiscoveryApp {
    pub app_id: u32,
    pub anonymous_principal: String,
}

// ── Promotion (Discovery Monetization) ─────────────────────────────────────

#[derive(CandidType, Deserialize, Clone, Serialize, Debug)]
pub struct PromotionConfig {
    pub min_bid_e8s: Nat,
    pub bid_increment_e8s: Nat,
    pub locked_period_ns: u64,
    pub feature_duration_ns: u64,
    pub ledger_canister: Principal,
    pub treasury: Principal,
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug)]
pub struct FeaturedSlot {
    pub app_id: u32,
    pub bidder: Principal,
    pub bid_amount_e8s: Nat,
    pub bid_time_ns: u64,
    pub locked_until_ns: u64,
    pub expires_at_ns: u64,
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug)]
pub struct HistoricalBid {
    pub app_id: u32,
    pub bidder: Principal,
    pub bid_amount_e8s: Nat,
    pub bid_time_ns: u64,
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug)]
pub struct PromotionStatus {
    pub config: PromotionConfig,
    pub featured: Option<FeaturedSlot>,
    pub min_next_bid_e8s: Nat,
    pub locked: bool,
    pub now_ns: u64,
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug)]
pub struct PlaceBidArg {
    pub app_id: u32,
    pub amount_e8s: Nat,
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug)]
pub enum PlaceBidError {
    Locked { until_ns: u64 },
    BelowFloor { floor_e8s: Nat },
    BelowIncrement { required_e8s: Nat },
    UnknownApp,
    TransferFailed(String),
    NotConfigured,
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug)]
pub enum PlaceBidResult {
    Ok(FeaturedSlot),
    Err(PlaceBidError),
}

// Minimal ICRC2 types for inter-canister transfer_from.

#[derive(CandidType, Deserialize, Clone, Debug)]
struct IcrcAccount {
    owner: Principal,
    subaccount: Option<Vec<u8>>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct Icrc2TransferFromArgs {
    spender_subaccount: Option<Vec<u8>>,
    from: IcrcAccount,
    to: IcrcAccount,
    amount: Nat,
    fee: Option<Nat>,
    memo: Option<Vec<u8>>,
    created_at_time: Option<u64>,
}

#[derive(CandidType, Deserialize, Debug)]
enum Icrc2TransferFromError {
    BadFee { expected_fee: Nat },
    BadBurn { min_burn_amount: Nat },
    InsufficientFunds { balance: Nat },
    InsufficientAllowance { allowance: Nat },
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    Duplicate { duplicate_of: Nat },
    TemporarilyUnavailable,
    GenericError { error_code: Nat, message: String },
}

#[derive(CandidType, Deserialize, Debug)]
enum Icrc2TransferFromResult {
    Ok(Nat),
    Err(Icrc2TransferFromError),
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
    // Key: root_id, Value: app id -> anonymous principal (text) the user uses for that app.
    pub static DISCOVERY_USER_PRINCIPALS: RefCell<HashMap<String, HashMap<u32, String>>> = RefCell::new(HashMap::new());
    pub static PROMOTION_CONFIG: RefCell<Option<PromotionConfig>> = RefCell::new(None);
    pub static FEATURED_SLOT: RefCell<Option<FeaturedSlot>> = RefCell::new(None);
    pub static BID_HISTORY: RefCell<Vec<HistoricalBid>> = RefCell::new(Vec::new());
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

fn find_app_for_request(registry: &std::collections::HashSet<DiscoveryApp>, request: &DiscoveryVisitRequest) -> Option<DiscoveryApp> {
    registry.iter().find(|app| match &request.derivation_origin {
        Some(req_do) => app.derivation_origin.as_deref() == Some(req_do.as_str()),
        None => app.hostname == request.hostname && app.derivation_origin.is_none(),
    }).cloned()
}

/// Tracks a visit to a dapp by hostname. Updates unique_users, is_global, is_anonymous.
/// Creates a new DiscoveryApp entry if none exists for the given derivation_origin (or hostname).
#[update]
pub async fn store_discovery_app(request: DiscoveryVisitRequest) {
    let root_id = get_root_id().await;

    let mut app = DISCOVERY_REGISTRY.with(|registry| {
        find_app_for_request(&registry.borrow(), &request)
    })
    .unwrap_or_else(|| {
        let new_id = DISCOVERY_REGISTRY.with(|registry| registry.borrow().len() as u32 + 1);
        DiscoveryApp {
            id: new_id,
            derivation_origin: request.derivation_origin.clone(),
            hostname: request.hostname.clone(),
            url: None,
            name: None,
            image: None,
            desc: None,
            is_global: false,
            is_anonymous: false,
            unique_users: 0,
            status: DiscoveryStatus::New,
        }
    });

    let is_new_visitor = DISCOVERY_VISITORS.with(|visitors| {
        visitors
            .borrow_mut()
            .entry(app.id)
            .or_insert_with(HashSet::new)
            .insert(root_id.clone())
    });

    if is_new_visitor {
        app.unique_users += 1;
    }

    match request.login {
        LoginType::Global if !app.is_global => app.is_global = true,
        LoginType::Anonymous if !app.is_anonymous => app.is_anonymous = true,
        _ => {}
    }

    if let Some(anon) = request.anonymous_principal {
        DISCOVERY_USER_PRINCIPALS.with(|map| {
            map.borrow_mut()
                .entry(root_id)
                .or_insert_with(HashMap::new)
                .insert(app.id, anon.to_text());
        });
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
        find_app_for_request(&registry.borrow(), &request)
    });

    let Some(app) = found_app else { return true };

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

    let needs_flag_flip = match request.login {
        LoginType::Global if !app.is_global => true,
        LoginType::Anonymous if !app.is_anonymous => true,
        _ => false,
    };

    if needs_flag_flip {
        return true;
    }

    // Backfill: if the caller passed an anonymous principal that we don't yet have
    // recorded for this app, signal that a follow-up store_discovery_app call is needed.
    if let Some(anon) = &request.anonymous_principal {
        let already_recorded = DISCOVERY_USER_PRINCIPALS.with(|map| {
            map.borrow()
                .get(&visitor_id)
                .and_then(|apps| apps.get(&app.id))
                .map(|p| p == &anon.to_text())
                .unwrap_or(false)
        });
        if !already_recorded {
            return true;
        }
    }

    false
}

#[query]
pub fn count_discovery_apps() -> u64 {
    DISCOVERY_REGISTRY.with(|registry| {
        registry.borrow().len() as u64
    })
}

/// Returns the (app_id, anonymous_principal) pairs recorded for the calling user.
/// Requires an inter-canister call to the identity manager to resolve the caller's
/// root id, hence #[update].
#[update]
pub async fn get_my_discovery_apps() -> Vec<UserDiscoveryApp> {
    let root_id = get_root_id().await;
    DISCOVERY_USER_PRINCIPALS.with(|map| {
        map.borrow()
            .get(&root_id)
            .map(|apps| {
                apps.iter()
                    .map(|(id, p)| UserDiscoveryApp {
                        app_id: *id,
                        anonymous_principal: p.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default()
    })
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

// ── Promotion endpoints ───────────────────────────────────────────────────

/// Configures the promotion module. Admin-only. Idempotent — overwrites
/// existing config. Must be called once before `place_bid` becomes usable.
#[update]
pub async fn set_promotion_config(config: PromotionConfig) {
    trap_if_not_authenticated_admin();
    PROMOTION_CONFIG.with(|c| *c.borrow_mut() = Some(config));
}

/// Returns the effective promotion status. Expired slots are reported as
/// `featured = None` without mutating storage.
#[query]
pub fn get_promotion_status() -> PromotionStatus {
    let cfg = PROMOTION_CONFIG
        .with(|c| c.borrow().clone())
        .expect("promotion not configured");
    let now = ic_cdk::api::time();
    let effective = FEATURED_SLOT
        .with(|s| s.borrow().clone())
        .filter(|s| s.expires_at_ns > now);
    let locked = effective
        .as_ref()
        .map(|s| s.locked_until_ns > now)
        .unwrap_or(false);
    let min_next_bid_e8s = match &effective {
        Some(s) => s.bid_amount_e8s.clone() + cfg.bid_increment_e8s.clone(),
        None => cfg.min_bid_e8s.clone(),
    };
    PromotionStatus {
        config: cfg,
        featured: effective,
        min_next_bid_e8s,
        locked,
        now_ns: now,
    }
}

/// Places a bid for the Featured slot. Pulls `amount_e8s` from the caller
/// via ICRC2 transfer_from on the configured ledger; on success, replaces
/// the slot with a fresh winner and resets both timers.
#[update]
pub async fn place_bid(arg: PlaceBidArg) -> PlaceBidResult {
    let cfg = match PROMOTION_CONFIG.with(|c| c.borrow().clone()) {
        Some(c) => c,
        None => return PlaceBidResult::Err(PlaceBidError::NotConfigured),
    };
    let now = ic_cdk::api::time();
    let bidder = ic_cdk::caller();

    let app_exists =
        DISCOVERY_REGISTRY.with(|r| r.borrow().iter().any(|a| a.id == arg.app_id));
    if !app_exists {
        return PlaceBidResult::Err(PlaceBidError::UnknownApp);
    }

    let current = FEATURED_SLOT
        .with(|s| s.borrow().clone())
        .filter(|s| s.expires_at_ns > now);

    let min_required = match &current {
        Some(s) if s.locked_until_ns > now => {
            return PlaceBidResult::Err(PlaceBidError::Locked {
                until_ns: s.locked_until_ns,
            });
        }
        Some(s) => s.bid_amount_e8s.clone() + cfg.bid_increment_e8s.clone(),
        None => cfg.min_bid_e8s.clone(),
    };
    if arg.amount_e8s < min_required {
        return PlaceBidResult::Err(if current.is_some() {
            PlaceBidError::BelowIncrement {
                required_e8s: min_required,
            }
        } else {
            PlaceBidError::BelowFloor {
                floor_e8s: min_required,
            }
        });
    }

    let transfer_args = Icrc2TransferFromArgs {
        spender_subaccount: None,
        from: IcrcAccount {
            owner: bidder,
            subaccount: None,
        },
        to: IcrcAccount {
            owner: cfg.treasury,
            subaccount: None,
        },
        amount: arg.amount_e8s.clone(),
        fee: None,
        memo: None,
        created_at_time: None,
    };
    let transfer_call: CallResult<(Icrc2TransferFromResult,)> =
        call(cfg.ledger_canister, "icrc2_transfer_from", (transfer_args,)).await;
    match transfer_call {
        Ok((Icrc2TransferFromResult::Ok(_),)) => {}
        Ok((Icrc2TransferFromResult::Err(e),)) => {
            return PlaceBidResult::Err(PlaceBidError::TransferFailed(format!("{e:?}")));
        }
        Err((_, msg)) => {
            return PlaceBidResult::Err(PlaceBidError::TransferFailed(msg));
        }
    }

    let slot = FeaturedSlot {
        app_id: arg.app_id,
        bidder,
        bid_amount_e8s: arg.amount_e8s.clone(),
        bid_time_ns: now,
        locked_until_ns: now + cfg.locked_period_ns,
        expires_at_ns: now + cfg.feature_duration_ns,
    };
    FEATURED_SLOT.with(|s| *s.borrow_mut() = Some(slot.clone()));
    BID_HISTORY.with(|h| {
        h.borrow_mut().push(HistoricalBid {
            app_id: slot.app_id,
            bidder: slot.bidder,
            bid_amount_e8s: slot.bid_amount_e8s.clone(),
            bid_time_ns: slot.bid_time_ns,
        })
    });

    PlaceBidResult::Ok(slot)
}

/// Clears the current featured slot unconditionally. Admin-only.
/// One-shot: history is preserved, no deny-list is created — the same
/// app can be re-promoted immediately by a fresh bid.
#[update]
pub fn veto_current_featured() {
    trap_if_not_authenticated_admin();
    FEATURED_SLOT.with(|s| *s.borrow_mut() = None);
}

#[query]
pub fn count_bid_history() -> u64 {
    BID_HISTORY.with(|h| h.borrow().len() as u64)
}

#[query]
pub fn get_bid_history_paginated(offset: u64, limit: u64) -> Vec<HistoricalBid> {
    BID_HISTORY.with(|h| {
        h.borrow()
            .iter()
            .skip(offset as usize)
            .take(limit as usize)
            .cloned()
            .collect()
    })
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
    discovery_user_principals: Option<HashMap<String, HashMap<u32, String>>>,
    promotion_config: Option<PromotionConfig>,
    featured_slot: Option<FeaturedSlot>,
    bid_history: Option<Vec<HistoricalBid>>,
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
    let discovery_user_principals = DISCOVERY_USER_PRINCIPALS.with(|m| m.borrow().clone());
    let promotion_config = PROMOTION_CONFIG.with(|c| c.borrow().clone());
    let featured_slot = FEATURED_SLOT.with(|s| s.borrow().clone());
    let bid_history = BID_HISTORY.with(|h| h.borrow().clone());
    let mem = Memory {
        registry,
        config,
        neurons: Some(neurons),
        discovery_apps: Some(discovery_apps),
        discovery_visitors: Some(discovery_visitors),
        discovery_user_principals: Some(discovery_user_principals),
        promotion_config,
        featured_slot,
        bid_history: Some(bid_history),
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
    DISCOVERY_USER_PRINCIPALS.with(|m| {
        *m.borrow_mut() = mo.discovery_user_principals.unwrap_or_default();
    });
    PROMOTION_CONFIG.with(|c| {
        *c.borrow_mut() = mo.promotion_config;
    });
    FEATURED_SLOT.with(|s| {
        *s.borrow_mut() = mo.featured_slot;
    });
    BID_HISTORY.with(|h| {
        *h.borrow_mut() = mo.bid_history.unwrap_or_default();
    });
}

#[test]
fn sub_account_test() {}

#[cfg(test)]
mod discovery_dedup_tests {
    use super::*;
    use std::collections::HashSet;

    fn make_app(id: u32, hostname: &str, derivation_origin: Option<&str>) -> DiscoveryApp {
        DiscoveryApp {
            id,
            hostname: hostname.to_string(),
            derivation_origin: derivation_origin.map(String::from),
            url: None,
            name: None,
            image: None,
            desc: None,
            is_global: false,
            is_anonymous: false,
            unique_users: 0,
            status: DiscoveryStatus::New,
        }
    }

    fn make_request(hostname: &str, derivation_origin: Option<&str>) -> DiscoveryVisitRequest {
        DiscoveryVisitRequest {
            hostname: hostname.to_string(),
            derivation_origin: derivation_origin.map(String::from),
            login: LoginType::Global,
        }
    }

    fn registry_from(apps: Vec<DiscoveryApp>) -> HashSet<DiscoveryApp> {
        apps.into_iter().collect()
    }

    // Same hostname, different derivation_origins → each request resolves to its own app.
    #[test]
    fn find_by_derivation_origin_ignores_hostname_collision() {
        let shared_host = "https://7p3gx-jaaaa-aaaal-acbda-cai.ic0.app";
        let app1 = make_app(1, shared_host, Some("https://5pati-hyaaa-aaaal-qb3yq-cai.raw.icp.io"));
        let app2 = make_app(2, shared_host, Some("https://awcae-maaaa-aaaam-abmyq-cai.icp0.io"));
        let registry = registry_from(vec![app1.clone(), app2.clone()]);

        let req1 = make_request(shared_host, Some("https://5pati-hyaaa-aaaal-qb3yq-cai.raw.icp.io"));
        let req2 = make_request(shared_host, Some("https://awcae-maaaa-aaaam-abmyq-cai.icp0.io"));

        assert_eq!(find_app_for_request(&registry, &req1).map(|a| a.id), Some(1));
        assert_eq!(find_app_for_request(&registry, &req2).map(|a| a.id), Some(2));
    }

    // No derivation_origin → fall back to hostname match (only when app also has no derivation_origin).
    #[test]
    fn find_by_hostname_when_no_derivation_origin() {
        let app = make_app(1, "https://nfid.one", None);
        let registry = registry_from(vec![app]);

        let req = make_request("https://nfid.one", None);
        assert_eq!(find_app_for_request(&registry, &req).map(|a| a.id), Some(1));
    }

    // Request with derivation_origin must NOT match an app that only has a hostname entry.
    #[test]
    fn derivation_origin_request_does_not_match_hostname_only_app() {
        let shared_host = "https://7p3gx-jaaaa-aaaal-acbda-cai.ic0.app";
        let app = make_app(1, shared_host, None);
        let registry = registry_from(vec![app]);

        let req = make_request(shared_host, Some("https://some-other-origin.icp0.io"));
        assert!(find_app_for_request(&registry, &req).is_none());
    }

    // Request without derivation_origin must NOT match an app that has one.
    #[test]
    fn no_derivation_origin_request_does_not_match_app_with_derivation_origin() {
        let shared_host = "https://7p3gx-jaaaa-aaaal-acbda-cai.ic0.app";
        let app = make_app(1, shared_host, Some("https://pingow.xyz"));
        let registry = registry_from(vec![app]);

        let req = make_request(shared_host, None);
        assert!(find_app_for_request(&registry, &req).is_none());
    }
}

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
