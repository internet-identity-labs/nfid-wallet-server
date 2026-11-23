//! External calls the marketplace makes: ICP ledger (ICRC-1/2) and NFT
//! collections (ICRC-7/37). Behind a trait so the purchase state machine can
//! be unit tested without a replica.

use candid::{CandidType, Deserialize, Nat, Principal};

use super::types::Account;

/// Result of a transfer on a ledger or NFT canister.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransferOutcome {
    /// Done. Holds the block index (also returned for a duplicate).
    Ok(Nat),
    /// The canister processed the request and refused it. Nothing moved.
    Rejected(String),
    /// The ledger expects another fee. Nothing moved.
    BadFee(Nat),
    /// Unknown whether it executed (call failed, temporarily unavailable…).
    /// Safe to retry with the same `created_at_time`.
    Unknown(String),
}

pub struct IcpTransferFrom {
    pub from: Account,
    pub to: Account,
    pub amount: Nat,
    pub fee: Nat,
    pub memo: Vec<u8>,
    pub created_at_time: u64,
}

pub struct IcpTransfer {
    pub from_subaccount: Option<Vec<u8>>,
    pub to: Account,
    pub amount: Nat,
    pub fee: Nat,
    pub memo: Vec<u8>,
    pub created_at_time: u64,
}

pub struct NftTransferFrom {
    pub from: Account,
    pub to: Account,
    pub token_id: Nat,
    pub memo: Vec<u8>,
    pub created_at_time: u64,
}

#[allow(async_fn_in_trait)]
pub trait Env {
    fn now(&self) -> u64;
    fn self_id(&self) -> Principal;

    async fn icp_fee(&self, ledger: Principal) -> Result<Nat, String>;
    async fn icp_transfer_from(&self, ledger: Principal, args: IcpTransferFrom) -> TransferOutcome;
    async fn icp_transfer(&self, ledger: Principal, args: IcpTransfer) -> TransferOutcome;

    async fn nft_owner_of(
        &self,
        collection: Principal,
        token_id: Nat,
    ) -> Result<Option<Account>, String>;
    /// Whether `spender` may move `token_id` out of the owner's default account.
    async fn nft_is_approved(
        &self,
        collection: Principal,
        token_id: Nat,
        spender: Account,
    ) -> Result<bool, String>;
    async fn nft_transfer_from(
        &self,
        collection: Principal,
        args: NftTransferFrom,
    ) -> TransferOutcome;
}

// ── IC implementation ──────────────────────────────────────────────────────

pub struct IcEnv;

#[derive(CandidType, Deserialize)]
struct Icrc2TransferFromArgs {
    spender_subaccount: Option<Vec<u8>>,
    from: Account,
    to: Account,
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

#[derive(CandidType, Deserialize)]
struct Icrc1TransferArgs {
    from_subaccount: Option<Vec<u8>>,
    to: Account,
    amount: Nat,
    fee: Option<Nat>,
    memo: Option<Vec<u8>>,
    created_at_time: Option<u64>,
}

#[derive(CandidType, Deserialize, Debug)]
enum Icrc1TransferError {
    BadFee { expected_fee: Nat },
    BadBurn { min_burn_amount: Nat },
    InsufficientFunds { balance: Nat },
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    Duplicate { duplicate_of: Nat },
    TemporarilyUnavailable,
    GenericError { error_code: Nat, message: String },
}

#[derive(CandidType, Deserialize)]
struct Icrc37IsApprovedArg {
    spender: Account,
    from_subaccount: Option<Vec<u8>>,
    token_id: Nat,
}

#[derive(CandidType, Deserialize)]
struct Icrc37TransferFromArg {
    spender_subaccount: Option<Vec<u8>>,
    from: Account,
    to: Account,
    token_id: Nat,
    memo: Option<Vec<u8>>,
    created_at_time: Option<u64>,
}

#[derive(CandidType, Deserialize, Debug)]
enum Icrc37TransferFromError {
    InvalidRecipient,
    Unauthorized,
    NonExistingTokenId,
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    Duplicate { duplicate_of: Nat },
    GenericError { error_code: Nat, message: String },
    GenericBatchError { error_code: Nat, message: String },
}

fn map_icrc2_error(e: Icrc2TransferFromError) -> TransferOutcome {
    match e {
        Icrc2TransferFromError::Duplicate { duplicate_of } => TransferOutcome::Ok(duplicate_of),
        Icrc2TransferFromError::BadFee { expected_fee } => TransferOutcome::BadFee(expected_fee),
        Icrc2TransferFromError::TemporarilyUnavailable => {
            TransferOutcome::Unknown("TemporarilyUnavailable".into())
        }
        other => TransferOutcome::Rejected(format!("{other:?}")),
    }
}

fn map_icrc1_error(e: Icrc1TransferError) -> TransferOutcome {
    match e {
        Icrc1TransferError::Duplicate { duplicate_of } => TransferOutcome::Ok(duplicate_of),
        Icrc1TransferError::BadFee { expected_fee } => TransferOutcome::BadFee(expected_fee),
        Icrc1TransferError::TemporarilyUnavailable => {
            TransferOutcome::Unknown("TemporarilyUnavailable".into())
        }
        other => TransferOutcome::Rejected(format!("{other:?}")),
    }
}

fn map_icrc37_error(e: Icrc37TransferFromError) -> TransferOutcome {
    match e {
        Icrc37TransferFromError::Duplicate { duplicate_of } => TransferOutcome::Ok(duplicate_of),
        // The token cannot be moved by us to that recipient.
        Icrc37TransferFromError::Unauthorized
        | Icrc37TransferFromError::NonExistingTokenId
        | Icrc37TransferFromError::InvalidRecipient => TransferOutcome::Rejected(format!("{e:?}")),
        // Timing, rate limits (ORIGYN reports them as GenericError) and batch
        // errors may pass on a later attempt.
        other => TransferOutcome::Unknown(format!("{other:?}")),
    }
}

fn reject_msg(code: ic_cdk::api::call::RejectionCode, msg: String) -> String {
    format!("{code:?}: {msg}")
}

impl Env for IcEnv {
    fn now(&self) -> u64 {
        ic_cdk::api::time()
    }

    fn self_id(&self) -> Principal {
        ic_cdk::id()
    }

    async fn icp_fee(&self, ledger: Principal) -> Result<Nat, String> {
        ic_cdk::call::<(), (Nat,)>(ledger, "icrc1_fee", ())
            .await
            .map(|(fee,)| fee)
            .map_err(|(c, m)| reject_msg(c, m))
    }

    async fn icp_transfer_from(&self, ledger: Principal, a: IcpTransferFrom) -> TransferOutcome {
        let args = Icrc2TransferFromArgs {
            spender_subaccount: None,
            from: a.from,
            to: a.to,
            amount: a.amount,
            fee: Some(a.fee),
            memo: Some(a.memo),
            created_at_time: Some(a.created_at_time),
        };
        match ic_cdk::call::<_, (Result<Nat, Icrc2TransferFromError>,)>(
            ledger,
            "icrc2_transfer_from",
            (args,),
        )
        .await
        {
            Ok((Ok(block),)) => TransferOutcome::Ok(block),
            Ok((Err(e),)) => map_icrc2_error(e),
            Err((c, m)) => TransferOutcome::Unknown(reject_msg(c, m)),
        }
    }

    async fn icp_transfer(&self, ledger: Principal, a: IcpTransfer) -> TransferOutcome {
        let args = Icrc1TransferArgs {
            from_subaccount: a.from_subaccount,
            to: a.to,
            amount: a.amount,
            fee: Some(a.fee),
            memo: Some(a.memo),
            created_at_time: Some(a.created_at_time),
        };
        match ic_cdk::call::<_, (Result<Nat, Icrc1TransferError>,)>(
            ledger,
            "icrc1_transfer",
            (args,),
        )
        .await
        {
            Ok((Ok(block),)) => TransferOutcome::Ok(block),
            Ok((Err(e),)) => map_icrc1_error(e),
            Err((c, m)) => TransferOutcome::Unknown(reject_msg(c, m)),
        }
    }

    async fn nft_owner_of(
        &self,
        collection: Principal,
        token_id: Nat,
    ) -> Result<Option<Account>, String> {
        ic_cdk::call::<_, (Vec<Option<Account>>,)>(collection, "icrc7_owner_of", (vec![token_id],))
            .await
            .map_err(|(c, m)| reject_msg(c, m))
            .map(|(owners,)| owners.into_iter().next().flatten())
    }

    async fn nft_is_approved(
        &self,
        collection: Principal,
        token_id: Nat,
        spender: Account,
    ) -> Result<bool, String> {
        let arg = Icrc37IsApprovedArg {
            spender,
            from_subaccount: None,
            token_id,
        };
        ic_cdk::call::<_, (Vec<bool>,)>(collection, "icrc37_is_approved", (vec![arg],))
            .await
            .map_err(|(c, m)| reject_msg(c, m))
            .map(|(res,)| res.first().copied().unwrap_or(false))
    }

    async fn nft_transfer_from(
        &self,
        collection: Principal,
        a: NftTransferFrom,
    ) -> TransferOutcome {
        let arg = Icrc37TransferFromArg {
            spender_subaccount: None,
            from: a.from,
            to: a.to,
            token_id: a.token_id,
            memo: Some(a.memo),
            created_at_time: Some(a.created_at_time),
        };
        // The ICRC-37 spec returns `vec opt TransferFromResult`. Some
        // implementations (ORIGYN core_nft) wrap it in an outer Result.
        let raw = ic_cdk::api::call::call_raw(
            collection,
            "icrc37_transfer_from",
            candid::encode_one(vec![arg]).expect("encode icrc37_transfer_from"),
            0,
        )
        .await;
        let bytes = match raw {
            Ok(b) => b,
            Err((c, m)) => return TransferOutcome::Unknown(reject_msg(c, m)),
        };
        type Item = Option<Result<Nat, Icrc37TransferFromError>>;
        let items: Vec<Item> = if let Ok(v) = candid::decode_one::<Vec<Item>>(&bytes) {
            v
        } else {
            match candid::decode_one::<Result<Vec<Item>, Icrc37TransferFromError>>(&bytes) {
                Ok(Ok(v)) => v,
                Ok(Err(e)) => return map_icrc37_error(e),
                Err(e) => return TransferOutcome::Unknown(format!("decode: {e}")),
            }
        };
        match items.into_iter().next().flatten() {
            Some(Ok(block)) => TransferOutcome::Ok(block),
            Some(Err(e)) => map_icrc37_error(e),
            None => TransferOutcome::Unknown("empty icrc37_transfer_from result".into()),
        }
    }
}
