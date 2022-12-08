#!/usr/bin/env bash
#dfx stop
#set -e
#trap 'dfx stop' EXIT

echo "===========SETUP========="
export IC_VERSION=dd3a710b03bd3ae10368a91b255571d012d1ec2f
gunzip ledger.wasm.gz
test -f ledger.wasm.gz ||curl -o ledger.wasm.gz "https://download.dfinity.systems/ic/$IC_VERSION/canisters/ledger-canister_notify-method.wasm.gz"
test -f ledger.wasm || gunzip ledger.wasm.gz
test -f ledger.private.did || curl -o ledger.private.did "https://raw.githubusercontent.com/dfinity/ic/$IC_VERSION/rs/rosetta-api/ledger.did"

test -f ledger.public.did ||curl -o ledger.public.did "https://raw.githubusercontent.com/dfinity/ic/$IC_VERSION/rs/rosetta-api/ledger_canister/ledger.did"

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

