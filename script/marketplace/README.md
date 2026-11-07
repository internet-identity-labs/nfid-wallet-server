# ICRC-7 marketplace (icrc1_oracle)

NFT marketplace inside `icrc1_oracle`, paid in ICP. Code: `src/icrc1_oracle/src/marketplace/`.

## Flow

1. **Collection.** Anyone registers an ICRC-7 collection with `store_nft_collection`, and it starts as `Community`. An admin can set `Verified`, `Spam` or `Hidden`. Only `Verified` and `Community` collections can be traded.
2. **Listing.**
   1. The seller approves the oracle on the NFT with `icrc37_approve_tokens` (spender = oracle principal).
   2. The seller calls `create_listing`. The oracle checks `icrc7_owner_of` and `icrc37_is_approved`.
   3. To change the price, cancel the listing and list again.
3. **Purchase.**
   1. The buyer calls `icrc2_approve` on the ICP ledger for `price + fee`.
   2. The buyer calls `buy_listing`. The oracle then:
      1. locks the listing;
      2. re-checks the owner and the approval;
      3. pulls `price` into its escrow subaccount (`get_marketplace_escrow`);
      4. moves the NFT to the buyer;
      5. pays the seller `price - fee`.
   3. If the NFT cannot be delivered, the buyer gets `price - fee` back.
4. **Retries.** Every transfer keeps its `created_at_time` and `memo`, so a repeated attempt comes back as a duplicate instead of moving funds twice.
   - A timer runs every 5 minutes and retries purchases that are still pending (`TxStatus::*Pending`).
   - After `max_retries` failed attempts, the purchase gets `needs_review` and the timer skips it.
   - An admin then runs `retry_marketplace_tx` or, after settling it by hand, `close_marketplace_tx`.

Listings and purchases are stored in stable structures (`ic-stable-structures`). Collections and config travel in the upgrade blob. The first upgrade from the old storage format (`storage::stable_save`) migrates the data automatically.

## Local testing

```bash
dfx start --clean --background
script/marketplace/setup_local.sh   # ICP ledger (ICRC-2), oracle, ORIGYN NFT collection, 5 NFTs for mkt_seller
script/marketplace/smoke_local.sh   # approve -> list -> approve ICP -> buy, checks balances and owner
cargo test -p icrc1_oracle marketplace
```

The test collection is a prebuilt `core_nft` wasm from [ORIGYN-SA/nft](https://github.com/ORIGYN-SA/nft) (ICRC-7 + ICRC-37), pinned by commit and checksum.

## Admin (admin_oracle)

```bash
cd admin_oracle
npx tsx admin_CLI.ts getNftCollections
npx tsx admin_CLI.ts formNftCollectionsCSV      # -> nft_collections.csv, edit status
npx tsx admin_CLI.ts uploadNftCollectionsCSV
npx tsx admin_CLI.ts setNftCollectionStatus <canister> Verified|Community|Spam|Hidden
npx tsx admin_CLI.ts removeNftCollection <canister>
npx tsx admin_CLI.ts setMarketplaceConfig prod|dev
npx tsx admin_CLI.ts getListings [collection]
npx tsx admin_CLI.ts cancelListing <id>
npx tsx admin_CLI.ts getMarketplaceTxs [limit]
npx tsx admin_CLI.ts getPendingTxs
npx tsx admin_CLI.ts retryTx <id> [--reset-dedup]   # reset only if the ledger shows the last attempt did not execute
npx tsx admin_CLI.ts closeTx <id> <note>
npx tsx admin_CLI.ts runMarketplaceClock
```

To point the CLI at a local replica, set `IC_HOST=http://127.0.0.1:8000` and `ORACLE_CANISTER_ID=$(dfx canister id icrc1_oracle)`.

`@icp-sdk/core` 5.x calls the `/api/v3` and `/api/v4` endpoints, which the dfx 0.28 local replica does not serve. Local runs need a newer replica or a proxy that rewrites the paths.
