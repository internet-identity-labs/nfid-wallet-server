#!/usr/bin/env ts-node
import {AdminManager} from "./admin_manager";

async function run() {
    const adminManager = new AdminManager();

    const method = process.argv[2];

    switch (method) {
        case 'formCSV':
            await adminManager.addICTokens();
            await adminManager.addToCSV();
            console.log('CSV ready!!!');
            break;
        case 'uploadCSV':
            await adminManager.addFromCSV();
            console.log('CSV uploaded!!!');
            break;
        case 'removeCanister':
            await adminManager.removeCanister(process.argv[3]);
            console.log('Canister removed!!!');
            break;
        case 'formNeuronsCSV':
            await adminManager.formNeuronsCSV();
            console.log('Neurons CSV ready!!!');
            break;
        case 'replaceNeurons':
            await adminManager.replaceNeuronsFromCSV();
            console.log('Neurons replaced!!!');
            break;
        default:
            console.log('Invalid method');
            break;
    }
}

run().catch(console.error);
