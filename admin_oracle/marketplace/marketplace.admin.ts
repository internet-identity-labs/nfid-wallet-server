import * as fs from "fs";
import { parse } from "json2csv";
import { parse as csvParse } from "csv-parse/sync";
import { ActorMethod } from "@icp-sdk/core/agent";
import { Principal } from "@icp-sdk/core/principal";
import type {
    Listing,
    ListingPage,
    MarketTx,
    MarketTxResult,
    ListingResult,
    MarketplaceConfig,
    NftCollection,
    NftCollectionStatus,
} from "../../test/idl/icrc1_oracle";
import { FILE_PATH_NFT_COLLECTIONS, MARKETPLACE_CONFIG, MarketplaceEnv } from "../constants";

const PAGE = 100n;

export const NFT_COLLECTION_STATUSES = ["Verified", "Community", "Spam", "Hidden"] as const;
export type NftCollectionStatusName = (typeof NFT_COLLECTION_STATUSES)[number];

interface NftCollectionCsvData {
    canister_id: string;
    name: string;
    symbol: string;
    status: string;
    logo: string | undefined;
    description: string | undefined;
    date_added: string;
}

export function statusName(status: NftCollectionStatus | { [k: string]: null }): string {
    return Object.keys(status)[0];
}

function toStatus(name: string): NftCollectionStatus {
    if (!NFT_COLLECTION_STATUSES.includes(name as NftCollectionStatusName)) {
        throw new Error(`Unknown collection status "${name}". Use one of: ${NFT_COLLECTION_STATUSES.join(", ")}`);
    }
    return { [name]: null } as NftCollectionStatus;
}

function unwrap<T>(result: MarketTxResult | ListingResult): T {
    if ("Err" in result) {
        throw new Error(`Marketplace error: ${JSON.stringify(result.Err)}`);
    }
    return result.Ok as T;
}

/** Admin operations for the ICRC-7 marketplace in icrc1_oracle. */
export class MarketplaceAdmin {
    constructor(private readonly actor: Record<string, ActorMethod>) {}

    // ── Collections ────────────────────────────────────────────────────

    async getCollections(): Promise<NftCollection[]> {
        const total = (await this.actor.count_nft_collections()) as bigint;
        const all: NftCollection[] = [];
        for (let offset = 0n; offset < total; offset += PAGE) {
            all.push(...((await this.actor.get_nft_collections_paginated(offset, PAGE)) as NftCollection[]));
        }
        return all;
    }

    /** Dumps collections to CSV for review; edit `status` and upload back. */
    async formCollectionsCSV(): Promise<number> {
        const collections = await this.getCollections();
        const rows: NftCollectionCsvData[] = collections.map((c) => ({
            canister_id: c.canister_id.toText(),
            name: c.name,
            symbol: c.symbol,
            status: statusName(c.status),
            logo: c.logo[0],
            description: c.description[0],
            date_added: c.date_added.toString(),
        }));
        const fields = ["canister_id", "name", "symbol", "status", "logo", "description", "date_added"];
        fs.writeFileSync(FILE_PATH_NFT_COLLECTIONS, parse(rows, { fields }));
        return rows.length;
    }

    /** Upserts collections from the CSV (statuses included). */
    async uploadCollectionsCSV(): Promise<number> {
        const records: NftCollectionCsvData[] = csvParse(fs.readFileSync(FILE_PATH_NFT_COLLECTIONS, "utf8"), {
            columns: true,
            skip_empty_lines: true,
        });
        const collections: NftCollection[] = records.map((r) => ({
            canister_id: Principal.fromText(r.canister_id),
            name: r.name,
            symbol: r.symbol,
            status: toStatus(r.status),
            logo: r.logo ? [r.logo] : [],
            description: r.description ? [r.description] : [],
            date_added: BigInt(r.date_added || Date.now() * 1_000_000),
        }));
        for (let i = 0; i < collections.length; i += 25) {
            await this.actor.replace_nft_collections(collections.slice(i, i + 25));
        }
        return collections.length;
    }

    async setCollectionStatus(canisterId: string, status: string): Promise<void> {
        await this.actor.set_nft_collection_status(Principal.fromText(canisterId), toStatus(status));
    }

    async removeCollection(canisterId: string): Promise<void> {
        await this.actor.remove_nft_collection(Principal.fromText(canisterId));
    }

    // ── Config ─────────────────────────────────────────────────────────

    async setConfig(env: MarketplaceEnv): Promise<void> {
        const config = MARKETPLACE_CONFIG[env];
        if (!config) {
            throw new Error(`Unknown marketplace env "${env}". Use "prod" or "dev".`);
        }
        await this.actor.set_marketplace_config(config);
    }

    async getConfig(): Promise<MarketplaceConfig | undefined> {
        return ((await this.actor.get_marketplace_config()) as [] | [MarketplaceConfig])[0];
    }

    // ── Listings ───────────────────────────────────────────────────────

    /** All listings on sale, newest first, optionally in one collection. */
    async getActiveListings(collection?: string): Promise<Listing[]> {
        const all: Listing[] = [];
        for (let offset = 0n; ; offset += PAGE) {
            const page = (await this.actor.get_listings({
                collection: collection ? [Principal.fromText(collection)] : [],
                sort: [{ Newest: null }],
                offset,
                limit: PAGE,
            })) as ListingPage;
            all.push(...page.items);
            if (BigInt(all.length) >= page.total || page.items.length === 0) return all;
        }
    }

    async cancelListing(listingId: bigint): Promise<Listing> {
        return unwrap<Listing>((await this.actor.cancel_listing(listingId)) as ListingResult);
    }

    // ── Purchases ──────────────────────────────────────────────────────

    async getTxs(limit: bigint): Promise<MarketTx[]> {
        return (await this.actor.get_marketplace_txs_paginated(0n, limit)) as MarketTx[];
    }

    /** Unfinished purchases; `needs_review` ones wait for retryTx / closeTx. */
    async getPendingTxs(): Promise<MarketTx[]> {
        return (await this.actor.get_pending_marketplace_txs()) as MarketTx[];
    }

    /**
     * Retries a stuck purchase. `resetDedup` gives the current transfer a new
     * created_at_time: use it only after checking on the ledger that the last
     * attempt did not go through (e.g. it failed with TooOld).
     */
    async retryTx(txId: bigint, resetDedup: boolean): Promise<MarketTx> {
        return unwrap<MarketTx>((await this.actor.retry_marketplace_tx(txId, resetDedup)) as MarketTxResult);
    }

    /** Closes a purchase that was settled by hand. */
    async closeTx(txId: bigint, note: string): Promise<MarketTx> {
        return unwrap<MarketTx>((await this.actor.close_marketplace_tx(txId, note)) as MarketTxResult);
    }

    async runClock(): Promise<number> {
        return (await this.actor.run_marketplace_clock()) as number;
    }
}
