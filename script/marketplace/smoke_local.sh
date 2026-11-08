#!/usr/bin/env bash
# End-to-end check of the marketplace on the local replica, using the
# mkt_seller / mkt_buyer identities from setup_local.sh:
#   seller approves the oracle on the NFT (ICRC-37) and lists it,
#   buyer approves ICP (ICRC-2) and buys,
#   NFT ends up with the buyer and the seller receives price - fee.
set -euo pipefail

cd "$(dirname "$0")/../.."

PRICE=100000000 # 1 ICP
FEE=10000

ORACLE="$(dfx canister id icrc1_oracle)"
SELLER="$(dfx identity get-principal --identity mkt_seller)"
BUYER="$(dfx identity get-principal --identity mkt_buyer)"
COLLECTION="$(dfx canister id nft_collection)"

log() { echo "==> $*"; }
fail() { echo "FAIL: $*" >&2; exit 1; }
icp_balance() {
  dfx canister call icp_ledger icrc1_balance_of "(record { owner = principal \"$1\"; subaccount = null })" \
    | grep -o '[0-9_]*' | head -1 | tr -d _
}

TOKEN_ID="$(dfx canister call nft_collection icrc7_tokens_of \
  "(record { owner = principal \"$SELLER\"; subaccount = null }, null, opt (1 : nat))" \
  | grep -o '[0-9_][0-9_]*' | head -1 | tr -d _)"
[[ -n "$TOKEN_ID" ]] || fail "seller owns no NFTs; run setup_local.sh"
log "Token $TOKEN_ID"

log "Seller approves the oracle on the NFT"
NOW_NS="$(($(date +%s) * 1000000000))"
dfx canister call --identity mkt_seller nft_collection icrc37_approve_tokens "(vec { record {
  token_id = $TOKEN_ID : nat;
  approval_info = record {
    spender = record { owner = principal \"$ORACLE\"; subaccount = null };
    from_subaccount = null; expires_at = null; memo = null;
    created_at_time = $NOW_NS : nat64;
  };
} })"

log "Seller lists it for $PRICE e8s"
OUT="$(dfx canister call --identity mkt_seller icrc1_oracle create_listing "(record {
  collection = principal \"$COLLECTION\"; token_id = $TOKEN_ID : nat; price_e8s = $PRICE : nat;
})")"
echo "$OUT"
LISTING_ID="$(echo "$OUT" | grep -o 'id = [0-9_]*' | head -1 | grep -o '[0-9_]*$' | tr -d _)"
[[ -n "$LISTING_ID" ]] || fail "create_listing failed"

SELLER_BEFORE="$(icp_balance "$SELLER")"
BUYER_BEFORE="$(icp_balance "$BUYER")"

log "Buyer approves price + fee for the oracle"
dfx canister call --identity mkt_buyer icp_ledger icrc2_approve "(record {
  spender = record { owner = principal \"$ORACLE\"; subaccount = null };
  amount = $((PRICE + FEE)) : nat;
})"

log "Buyer buys listing $LISTING_ID"
OUT="$(dfx canister call --identity mkt_buyer icrc1_oracle buy_listing "($LISTING_ID : nat64)")"
echo "$OUT"
echo "$OUT" | grep -q "Completed" || fail "purchase did not complete"

OWNER="$(dfx canister call nft_collection icrc7_owner_of "(vec { $TOKEN_ID : nat })")"
echo "$OWNER" | grep -q "$BUYER" || fail "NFT owner is not the buyer: $OWNER"

SELLER_AFTER="$(icp_balance "$SELLER")"
BUYER_AFTER="$(icp_balance "$BUYER")"
# Buyer pays approve fee + transfer_from fee on top of the price.
[[ $((SELLER_AFTER - SELLER_BEFORE)) -eq $((PRICE - FEE)) ]] || fail "seller got $((SELLER_AFTER - SELLER_BEFORE))"
[[ $((BUYER_BEFORE - BUYER_AFTER)) -eq $((PRICE + 2 * FEE)) ]] || fail "buyer paid $((BUYER_BEFORE - BUYER_AFTER))"
ESCROW="$(dfx canister call icp_ledger icrc1_balance_of "$(dfx canister call icrc1_oracle get_marketplace_escrow)" \
  | grep -o '[0-9_]*' | head -1 | tr -d _)"
[[ "$ESCROW" -eq 0 ]] || fail "escrow holds $ESCROW"

log "OK: NFT $TOKEN_ID moved to the buyer, seller received $((PRICE - FEE)) e8s, escrow empty"
