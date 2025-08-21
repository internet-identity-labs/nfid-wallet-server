#!/usr/bin/env ts-node
import { AdminManager } from "./admin_manager";

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
        default:
            console.log("Invalid method");
            break;
    }
}

run().catch(console.error);
