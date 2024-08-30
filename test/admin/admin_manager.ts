import {idlFactory as icrcOracle1Idl} from "../idl/icrc1_oracle_idl";
import {Ed25519KeyIdentity} from "@dfinity/identity";
import {SnsParser} from "./sns";
import {NativeParser} from "./native";
import {ChainFusionParser} from "./chain_fusion";
import {ActorMethod} from "@dfinity/agent";
import {ICRC1} from "../idl/icrc1_oracle";
import {parse} from 'json2csv';
import * as fs from 'fs';
import {parse as csvParse} from 'csv-parse/sync';
import {ICRC1CsvData} from "./types";
import {getActor, mapCategory, mapCategoryCSVToCategory} from "./util";
import {ChainFusionTestnetParser} from "./chain_fusion_testnet";
import {CANISTER_ID, FILE_PATH, KEY_PAIR} from "./constants";

export class AdminManager {

    private actor: Record<string, ActorMethod>

    constructor() {
        const adminEd = Ed25519KeyIdentity.fromParsedJson(KEY_PAIR);
        this.actor = getActor(CANISTER_ID, adminEd, icrcOracle1Idl);
    }

    async addICTokens() {
        const sns = await new SnsParser().parseCanister();
        const native = await new NativeParser().parseCanister();
        const chainFusion = await new ChainFusionParser().parseCanister();
        const chainFusionTestnet = await new ChainFusionTestnetParser().parseCanister();
        const all = native
            .concat(chainFusion)
            .concat(chainFusionTestnet)
            .concat(sns);
        await this.actor.store_new_icrc1_canisters(all);
    }

    async addToCSV() {
        const canisters_from_oracle = await this.actor.get_all_icrc1_canisters() as ICRC1[];
        const fields = ['name', 'symbol', 'ledger', 'index', 'category', 'logo'];
        const opts = {fields};
        try {
            const csv = parse(canisters_from_oracle
                    .map((c) => {
                        return {
                            name: c.name,
                            ledger: c.ledger,
                            category: mapCategory(c.category).toString(),
                            index: c.index.length > 0 ? c.index[0] : undefined,
                            symbol: c.symbol,
                            logo: c.logo.length > 0 ? c.logo[0] : undefined
                        }
                    })
                , opts);
            fs.writeFileSync(FILE_PATH, csv);
            console.log('CSV file saved successfully!');
        } catch (err) {
            console.error(err);
        }
    }

    async addFromCSV() {
        const csvData = fs.readFileSync(FILE_PATH, 'utf8');
        const records: ICRC1CsvData[] = csvParse(csvData, {
            columns: true,
            skip_empty_lines: true,
        });
        const asd: ICRC1[] = records
            .map((record) => {
                return {
                    name: record.name,
                    ledger: record.ledger,
                    category: mapCategoryCSVToCategory(record.category),
                    index: record.index === undefined ? [] : [record.index],
                    symbol: record.symbol,
                    logo: record.logo === undefined ? [] : [record.logo]
                }
            });
        await this.actor.replace_icrc1_canisters(asd);
    }
}
