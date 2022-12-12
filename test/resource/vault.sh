#!/usr/bin/env bash
echo "===========DEPLOY VAULT========="
LEDGER_ID="$(dfx canister id ledger)"

dfx deploy vault --argument "(opt record { ledger_canister_id=principal \"${LEDGER_ID}\";}, )"

VAULT_CANISTER_ID="$(dfx canister id vault)"
echo $VAULT_CANISTER_ID
VAULT_ACCOUNT_SUB_ID="$(dfx canister call vault sub '(1)')"
VAULT_ACCOUNT_SUB_BYTES="$(dfx canister call vault sub_bytes '(1)')"
VAULT_ACCOUNT_SUB_BYTES="$(dfx ledger account-id --of-canister vault)"

echo $VAULT_ACCOUNT_SUB_BYTES

TOKENS_TRANSFER_ACCOUNT_ID="$(dfx ledger account-id --of-canister vault)"
VAULT_ACCOUNT_SUB_ID="$(dfx canister call vault sub '(1)')"


TOKENS_TRANSFER_ACCOUNT_ID_BYTES="$(python3 -c 'print("vec{" + ";".join([str(b) for b in bytes.fromhex('$VAULT_ACCOUNT_SUB_ID')]) + "}")')"
dfx canister call ledger transfer "(record { to=${TOKENS_TRANSFER_ACCOUNT_ID_BYTES};  amount=record { e8s=200_000_000 }; fee=record { e8s=10_000 }; memo=0:nat64; }, )"

dfx canister call ledger account_balance '(record { account = '$(python3 -c 'print("vec{" + ";".join([str(b) for b in bytes.fromhex('$VAULT_ACCOUNT_SUB_ID')]) + "}")')' })'

echo "DONE"
