#!/usr/bin/env bash
echo "===========TARGET WALLET BALANCE========="
WALLET_ACCOUNT_SUB_ID="$(dfx canister call vault sub '(1)')"
WALLET_ACCOUNT_ID_BYTES="$(python3 -c 'print("vec{" + ";".join([str(b) for b in bytes.fromhex'$WALLET_ACCOUNT_SUB_ID']) + "}")')"
dfx canister call ledger account_balance '(record { account = '$(python3 -c 'print("vec{" + ";".join([str(b) for b in bytes.fromhex'$WALLET_ACCOUNT_ID_BYTES']) + "}")')' })'

echo "DONE"
