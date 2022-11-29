#!/usr/bin/env bash
echo "===========DEPLOY VAULT========="
LEDGER_ID="$(dfx canister id ledger)"

dfx deploy vault

VAULT_CANISTER_ID="$(dfx canister id vault)"
echo $VAULT_CANISTER_ID
VAULT_ACCOUNT_SUB_ID="$(dfx canister call vault sub '(1)')"
echo $VAULT_ACCOUNT_SUB_ID
VAULT_ACCOUNT_ID_BYTES="$(python3 -c 'print("vec{" + ";".join([str(b) for b in bytes.fromhex'$VAULT_ACCOUNT_SUB_ID']) + "}")')"


dfx canister call ledger transfer "(record { to=${VAULT_ACCOUNT_ID_BYTES}; amount=record { e8s=100_000 }; fee=record { e8s=10_000 }; memo=0:nat64; }, )"
dfx canister call ledger account_balance '(record { account = '$(python3 -c 'print("vec{" + ";".join([str(b) for b in bytes.fromhex'$VAULT_ACCOUNT_SUB_ID']) + "}")')' })'

echo "DONE"
