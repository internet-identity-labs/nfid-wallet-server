#!/usr/bin/env -S npx tsx
import { AdminManager } from "./admin_manager";
import { PromotionEnv } from "./constants";

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
        default:
            console.log("Invalid method");
            break;
    }
}

run().catch(console.error);
