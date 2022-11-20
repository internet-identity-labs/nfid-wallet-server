#!/usr/bin/env bash
echo "===========DEPLOY VAULT========="

dfx deploy vault --argument "(record { ledger_canister_id=principal \"${LEDGER_ID}\" }, )"

VAULT_ACCOUNT_ID="$(dfx ledger account-id --of-canister vault)"
VAULT_ACCOUNT_ID_BYTES="$(python3 -c 'print("vec{" + ";".join([str(b) for b in bytes.fromhex("'$VAULT_ACCOUNT_ID'")]) + "}")')"

dfx canister call ledger transfer "(record { to=${VAULT_ACCOUNT_ID_BYTES}; amount=record { e8s=100_000 }; fee=record { e8s=10_000 }; memo=0:nat64; }, )"
dfx canister call ledger account_balance '(record { account = '$(python3 -c 'print("vec{" + ";".join([str(b) for b in bytes.fromhex("'$VAULT_ACCOUNT_ID'")]) + "}")')' })'

echo "DONE"
