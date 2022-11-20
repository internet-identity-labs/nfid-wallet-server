#!/usr/bin/env bash
#dfx stop
#set -e
#trap 'dfx stop' EXIT

echo "===========SETUP========="
export IC_VERSION=2cb0afe1f49b8bbd4e60db234ca1f4a6f68ea115
test -f ledger.wasm.gz || curl -o ledger.wasm.gz https://download.dfinity.systems/ic/${IC_VERSION}/canisters/ledger-canister_notify-method.wasm.gz
test -f ledger.wasm || gunzip ledger.wasm.gz
test -f ledger.private.did || curl -o ledger.private.did https://raw.githubusercontent.com/dfinity/ic/${IC_VERSION}/rs/rosetta-api/ledger.did
test -f ledger.public.did || curl -o ledger.public.did https://raw.githubusercontent.com/dfinity/ic/${IC_VERSION}/rs/rosetta-api/ledger_canister/ledger.did
echo "===========START DFX========="
dfx start --background --clean
dfx identity new alice --disable-encryption || true
cat <<<"$(jq '.canisters.ledger.candid="ledger.private.did"' dfx.json)" >dfx.json
export MINT_ACC=$(dfx --identity anonymous ledger account-id)
export LEDGER_ACC=$(dfx ledger account-id)
export ARCHIVE_CONTROLLER=$(dfx identity get-principal)
echo "===========DEPLOY LEDGER========="
dfx deploy ledger --argument '(record {minting_account = "'${MINT_ACC}'"; initial_values = vec { record { "'${LEDGER_ACC}'"; record { e8s=100_000_000_000 } }; }; send_whitelist = vec {}})'
cat <<<"$(jq '.canisters.ledger.candid="ledger.public.did"' dfx.json)" >dfx.json
LEDGER_ID="$(dfx canister id ledger)"

echo "===========LEDGER SETUP DONE========="

echo "===========DEPLOY VAULT========="

dfx deploy vault --argument "(record { ledger_canister_id=principal \"${LEDGER_ID}\" }, )"

VAULT_CANISTER_ID="$(dfx canister id vault)"
echo $VAULT_CANISTER_ID
VAULT_ACCOUNT_SUB_ID="$(dfx canister call vault sub '("'${VAULT_CANISTER_ID}'", 1, 1)')"
echo $VAULT_ACCOUNT_SUB_ID
VAULT_ACCOUNT_ID_BYTES="$(python3 -c 'print("vec{" + ";".join([str(b) for b in bytes.fromhex'$VAULT_ACCOUNT_SUB_ID']) + "}")')"


dfx canister call ledger transfer "(record { to=${VAULT_ACCOUNT_ID_BYTES}; amount=record { e8s=100_000 }; fee=record { e8s=10_000 }; memo=0:nat64; }, )"
dfx canister call ledger account_balance '(record { account = '$(python3 -c 'print("vec{" + ";".join([str(b) for b in bytes.fromhex'$VAULT_ACCOUNT_SUB_ID']) + "}")')' })'

echo "DONE"
