#!/usr/bin/env bash
# Local environment for the ICRC-7 marketplace in icrc1_oracle.
#
# Deploys to the running local replica:
#   icp_ledger      ICP ledger with ICRC-2, balances for admin/seller/buyer
#   icrc1_oracle    marketplace, current dfx identity as operator
#   nft_collection  ORIGYN ICRC-7/37 collection with NFTs minted to the seller
# and registers the collection in the oracle as Verified.
#
# Usage:  dfx start --clean --background && script/marketplace/setup_local.sh
# Env:    SELLER / BUYER   principals to fund and mint to
#                          (default: dfx identities mkt_seller / mkt_buyer)
#         MINT_COUNT       NFTs to mint to the seller (default 5)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
WASM_DIR="$ROOT/script/marketplace/wasm"
cd "$ROOT"

ICP_LEDGER_URL="https://github.com/dfinity/ic/releases/download/ledger-suite-icp-2025-08-29/ledger-canister_notify-method.wasm.gz"
ICP_LEDGER_SHA="51f4be010f23064137defacd627ffbec024c5133210c68ca3b80ab8f257101d6"
CORE_NFT_URL="https://raw.githubusercontent.com/ORIGYN-SA/nft/cd4097938ee1e5da16e76134e0ce02812fb91723/src/core_nft/wasm/core_nft_canister.wasm.gz"
CORE_NFT_SHA="eca40f674d85bf292b1616f848d62422423c41c36dae02bfe907b0b4fa66c84e"
MINT_COUNT="${MINT_COUNT:-5}"

log() { echo "==> $*"; }

download() { # url sha256 dest
  if [[ ! -f "$3" ]] || [[ "$(shasum -a 256 "$3" | cut -d' ' -f1)" != "$2" ]]; then
    log "Downloading $(basename "$3")"
    curl -fsSL -o "$3" "$1"
  fi
  local got
  got="$(shasum -a 256 "$3" | cut -d' ' -f1)"
  if [[ "$got" != "$2" ]]; then
    echo "Checksum mismatch for $3: $got" >&2
    exit 1
  fi
}

identity_principal() { # name
  dfx identity list 2>/dev/null | grep -qx "$1" || dfx identity new "$1" --storage-mode plaintext >/dev/null 2>&1
  dfx identity get-principal --identity "$1"
}

account_id() { dfx ledger account-id --of-principal "$1"; }

if ! dfx ping >/dev/null 2>&1; then
  echo "Local replica is not running. Start it with: dfx start --clean --background" >&2
  exit 1
fi

mkdir -p "$WASM_DIR"
download "$ICP_LEDGER_URL" "$ICP_LEDGER_SHA" "$WASM_DIR/icp_ledger.wasm.gz"
download "$CORE_NFT_URL" "$CORE_NFT_SHA" "$WASM_DIR/core_nft.wasm.gz"

ADMIN="$(dfx identity get-principal)"
SELLER="${SELLER:-$(identity_principal mkt_seller)}"
BUYER="${BUYER:-$(identity_principal mkt_buyer)}"
MINTER="$(dfx identity get-principal --identity anonymous)"

log "ICP ledger (ICRC-2 enabled)"
dfx deploy icp_ledger --mode reinstall -y --argument "(variant { Init = record {
  minting_account = \"$(account_id "$MINTER")\";
  initial_values = vec {
    record { \"$(account_id "$ADMIN")\"; record { e8s = 1_000_000_000_000 : nat64 } };
    record { \"$(account_id "$SELLER")\"; record { e8s = 10_000_000_000 : nat64 } };
    record { \"$(account_id "$BUYER")\"; record { e8s = 10_000_000_000 : nat64 } };
  };
  send_whitelist = vec {};
  transfer_fee = opt record { e8s = 10_000 : nat64 };
  token_symbol = opt \"ICP\";
  token_name = opt \"Internet Computer (local)\";
  feature_flags = opt record { icrc2 = true };
} })"
ICP_LEDGER="$(dfx canister id icp_ledger)"

log "icrc1_oracle"
dfx deploy icrc1_oracle --mode reinstall -y --argument "(opt record {})"
ORACLE="$(dfx canister id icrc1_oracle)"
dfx canister call icrc1_oracle set_operator "(principal \"$ADMIN\")" >/dev/null
dfx canister call icrc1_oracle set_marketplace_config "(record {
  icp_ledger = principal \"$ICP_LEDGER\";
  min_price_e8s = 1_000_000 : nat;
  max_retries = 5 : nat32;
})" >/dev/null

log "NFT collection (ORIGYN core_nft)"
dfx deploy nft_collection --mode reinstall -y --argument "(variant { Init = record {
  name = \"NFID Test Collection\";
  symbol = \"NFIDT\";
  description = opt \"Local collection for marketplace testing\";
  logo = null;
  base_url = null;
  test_mode = true;
  permissions = record { user_permissions = vec {} };
  version = record { major = 0 : nat32; minor = 1 : nat32; patch = 0 : nat32 };
  commit_hash = \"local\";
  vetkd_key_name = \"dfx_test_key\";
  vetkd_context = \"nfid-marketplace-test\";
  collection_metadata = vec {};
  approval_init = record {
    max_approvals_per_token_or_collection = opt (10 : nat);
    max_revoke_approvals = opt (10 : nat);
  };
  supply_cap = null;
  tx_window = null;
  permitted_drift = null;
  storage_cycles = null;
  default_take_value = null;
  max_take_value = null;
  max_canister_storage_threshold = null;
  max_update_batch_size = null;
  max_query_batch_size = null;
  max_memo_size = null;
  atomic_batch_transfers = null;
} })"
COLLECTION="$(dfx canister id nft_collection)"

log "Minting $MINT_COUNT NFTs to the seller"
for i in $(seq 1 "$MINT_COUNT"); do
  dfx canister call nft_collection mint "(record { mint_requests = vec { record {
    token_owner = record { owner = principal \"$SELLER\"; subaccount = null };
    metadata = vec { record { \"icrc7:name\"; variant { Text = \"NFID Test #$i\" } } };
    memo = null;
    private_content = null;
    public_content = null;
  } } })" >/dev/null
done

log "Registering the collection in the oracle"
dfx canister call icrc1_oracle store_nft_collection "(record {
  canister_id = principal \"$COLLECTION\";
  name = \"NFID Test Collection\";
  symbol = \"NFIDT\";
  logo = null;
  description = opt \"Local collection for marketplace testing\";
})" >/dev/null
dfx canister call icrc1_oracle set_nft_collection_status \
  "(principal \"$COLLECTION\", variant { Verified })" >/dev/null

cat <<EOF

Marketplace local setup done.
  icp_ledger      $ICP_LEDGER
  icrc1_oracle    $ORACLE   (operator: $ADMIN)
  nft_collection  $COLLECTION
  seller          $SELLER   ($(dfx canister call nft_collection icrc7_balance_of "(vec { record { owner = principal \"$SELLER\"; subaccount = null } })" | tr -d '\n ' ) NFTs)
  buyer           $BUYER

Next: script/marketplace/smoke_local.sh runs list -> buy end to end.
EOF
