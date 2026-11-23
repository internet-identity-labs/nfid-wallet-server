#!/usr/bin/env -S npx tsx
import { AdminManager } from "./admin_manager";
import { MarketplaceEnv, PromotionEnv } from "./constants";
import { statusName } from "./marketplace";

function jsonStringify(value: unknown): string {
    return JSON.stringify(
        value,
        (_, v) => (typeof v === "bigint" ? v.toString() : v),
        2,
    );
}

async function run() {
    const adminManager = new AdminManager();

    const method = process.argv[2];

    switch (method) {
        case "formCSV":
            await adminManager.addICTokens();
            await adminManager.addToCSV();
            console.log("Tokens oracle has been generated!!!");
            break;
        case "uploadCSV":
            await adminManager.addFromCSV();
            console.log("Tokens oracle has been uploaded!!!");
            break;
        case "removeCanister":
            await adminManager.removeCanister(process.argv[3]);
            console.log("Token canister has been removed!!!");
            break;
        case "formNeuronsCSV":
            await adminManager.formNeuronsCSV();
            console.log("Neurons oracle has been generated!!!");
            break;
        case "replaceNeurons":
            await adminManager.replaceNeuronsFromCSV();
            console.log("Neuron to follow has been replaced!!!");
            break;
        case "formDiscoveryCSV":
            await adminManager.formDiscoveryCSV();
            console.log("Discovery apps CSV has been generated!!!");
            break;
        case "uploadDiscoveryCSV":
            await adminManager.replaceDiscoveryFromCSV();
            console.log("Discovery apps have been uploaded!!!");
            break;
        case "enrichDiscovery":
            await adminManager.enrichNewDiscoveryApps();
            console.log("New discovery apps have been enriched!!!");
            break;
        case "setPromotionConfig": {
            const env = process.argv[3] as PromotionEnv;
            await adminManager.setPromotionConfig(env);
            console.log(`Promotion config (${env}) has been uploaded!!!`);
            break;
        }
        case "vetoFeatured":
            await adminManager.vetoFeatured();
            console.log("Current featured slot has been cleared!!!");
            break;
        case "getFeatured": {
            const status = await adminManager.getPromotionStatus();
            console.log(jsonStringify(status));
            break;
        }
        case "getBidHistory": {
            const history = await adminManager.getBidHistory();
            console.log(jsonStringify(history));
            break;
        }
        // ── ICRC-7 marketplace ──────────────────────────────────────
        case "formNftCollectionsCSV": {
            const n = await adminManager.marketplace.formCollectionsCSV();
            console.log(`${n} NFT collections written to CSV!!!`);
            break;
        }
        case "uploadNftCollectionsCSV": {
            const n = await adminManager.marketplace.uploadCollectionsCSV();
            console.log(`${n} NFT collections have been uploaded!!!`);
            break;
        }
        case "getNftCollections": {
            const collections = await adminManager.marketplace.getCollections();
            console.table(collections.map((c) => ({
                canister_id: c.canister_id.toText(),
                name: c.name,
                symbol: c.symbol,
                status: statusName(c.status),
            })));
            break;
        }
        case "setNftCollectionStatus":
            await adminManager.marketplace.setCollectionStatus(process.argv[3], process.argv[4]);
            console.log("NFT collection status has been updated!!!");
            break;
        case "removeNftCollection":
            await adminManager.marketplace.removeCollection(process.argv[3]);
            console.log("NFT collection has been removed!!!");
            break;
        case "setMarketplaceConfig": {
            const env = process.argv[3] as MarketplaceEnv;
            await adminManager.marketplace.setConfig(env);
            console.log(`Marketplace config (${env}) has been uploaded!!!`);
            break;
        }
        case "getMarketplaceConfig":
            console.log(jsonStringify(await adminManager.marketplace.getConfig()));
            break;
        case "getListings":
            console.log(jsonStringify(await adminManager.marketplace.getActiveListings(process.argv[3])));
            break;
        case "cancelListing":
            console.log(jsonStringify(await adminManager.marketplace.cancelListing(BigInt(process.argv[3]))));
            break;
        case "getMarketplaceTxs":
            console.log(jsonStringify(await adminManager.marketplace.getTxs(BigInt(process.argv[3] ?? 50))));
            break;
        case "getPendingTxs":
            console.log(jsonStringify(await adminManager.marketplace.getPendingTxs()));
            break;
        case "retryTx":
            console.log(jsonStringify(await adminManager.marketplace.retryTx(
                BigInt(process.argv[3]),
                process.argv[4] === "--reset-dedup",
            )));
            break;
        case "closeTx":
            console.log(jsonStringify(await adminManager.marketplace.closeTx(
                BigInt(process.argv[3]),
                process.argv.slice(4).join(" "),
            )));
            break;
        case "runMarketplaceClock":
            console.log(`Processed ${await adminManager.marketplace.runClock()} pending purchase(s)`);
            break;
        default:
            console.log("Invalid method");
            break;
    }
}

run().catch(console.error);
