use ic_ledger_types::{AccountIdentifier, BlockIndex, DEFAULT_FEE, Memo, Subaccount, Tokens};

use crate::CONF;

pub async fn transfer(amount: u64, to: AccountIdentifier, from_subaccount: Subaccount) -> Result<BlockIndex, String> {
    let tokens = Tokens::from_e8s(amount);
    let ledger_canister_id = CONF.with(|conf| conf.borrow().ledger_canister_id);
    let transfer_args = ic_ledger_types::TransferArgs {
        memo: Memo(0),
        amount: tokens,
        fee: DEFAULT_FEE,
        from_subaccount: Some(from_subaccount),
        to,
        created_at_time: None,
    };
    ic_ledger_types::transfer(ledger_canister_id, transfer_args).await
        .map_err(|e| format!("failed to call ledger: {:?}", e))?
        .map_err(|e| format!("ledger transfer error {:?}", e))
}