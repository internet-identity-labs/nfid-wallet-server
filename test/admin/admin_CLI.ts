#!/usr/bin/env ts-node
import {AdminManager} from "./admin_manager";

async function run() {
    const adminManager = new AdminManager();

    const method = process.argv[2];

    switch (method) {
        case 'formCSV':
            await adminManager.addICTokens();
            await adminManager.addToCSV();
            console.log('Тёма, CSV готов!!!');
            break;
        case 'uploadCSV':
            await adminManager.addFromCSV();
            console.log('Тёма, загрузили!!! Полёт нормальный!!!');
            break;
        default:
            console.log('Тёма, не то написал!!!');
            break;
    }
}

run().catch(console.error);
