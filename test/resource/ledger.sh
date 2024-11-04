#!/usr/bin/env bash
echo "===========SETUP========="
export IC_VERSION=aba60ffbc46acfc8990bf4d5685c1360bd7026b9
gunzip ledger.wasm.gz
test -f ledger.wasm.gz ||curl -o ledger.wasm.gz "https://download.dfinity.systems/ic/$IC_VERSION/canisters/ledger-canister_notify-method.wasm.gz"
test -f ledger.wasm || gunzip ledger.wasm.gz
test -f ledger.private.did || curl -o ledger.private.did "https://raw.githubusercontent.com/dfinity/ic/$IC_VERSION/rs/rosetta-api/ledger.did"

test -f ledger.public.did ||curl -o ledger.public.did "https://raw.githubusercontent.com/dfinity/ic/$IC_VERSION/rs/rosetta-api/ledger_canister/ledger.did"

echo "===========START DFX========="
cat <<<"$(jq '.canisters.ledger.candid="ledger.private.did"' dfx.json)" > dfx.json
export MINT_ACC=$(dfx --identity anonymous ledger account-id)
export LEDGER_ACC=$(dfx ledger account-id)
export ARCHIVE_CONTROLLER=$(dfx identity get-principal)
echo "===========DEPLOY LEDGER========="
dfx deploy ledger --mode reinstall -y --argument '(record {minting_account = "'${MINT_ACC}'"; initial_values = vec { record { "'${LEDGER_ACC}'"; record { e8s=100_000_000_000 } }; }; send_whitelist = vec {}})'
cat <<<"$(jq '.canisters.ledger.candid="ledger.public.did"' dfx.json)" >dfx.json

