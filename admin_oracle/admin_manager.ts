import {idlFactory as icrcOracle1Idl} from "../test/idl/icrc1_oracle_idl";
import {Ed25519KeyIdentity} from "@dfinity/identity";
import {SnsParser} from "./sns";
import {NativeParser} from "./native";
import {ChainFusionParser} from "./chain_fusion";
import * as Agent from "@dfinity/agent";
import {ActorMethod, HttpAgent} from "@dfinity/agent";
import {ICRC1} from "../test/idl/icrc1_oracle";
import {parse} from 'json2csv';
import * as fs from 'fs';
import {parse as csvParse} from 'csv-parse/sync';
import {ICRC1CsvData, PairToSend} from "./types";
import {getActor, mapCategory, mapCategoryCSVToCategory} from "./util";
import {ChainFusionTestnetParser} from "./chain_fusion_testnet";
import {CANISTER_ID, FILE_PATH, KEY_PAIR, LEDGER_TO_SEND, LIST_TO_SEND_PATH} from "./constants";
import {getMetadata} from "./metadata_service";
import sharp from 'sharp'
import {_SERVICE as ICRC1ServiceIDL, Account, Icrc1TransferResult, TransferArg} from "./idl/icrc1.idl";
import {idlFactory as icrc1IDL} from "./idl/icrc1";
import {Principal} from "@dfinity/principal";
import { stakeNeuron } from "./sns-governance.api";

export class AdminManager {

    private actor: Record<string, ActorMethod>
    private adminED: Ed25519KeyIdentity

    constructor() {
        this.adminED = Ed25519KeyIdentity.fromParsedJson(KEY_PAIR);
        this.actor = getActor(CANISTER_ID, this.adminED, icrcOracle1Idl);
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

        const chunkArray = (arr: ICRC1[], chunkSize: number): ICRC1[][] => {
            const chunks: ICRC1[][] = [];
            for (let i = 0; i < arr.length; i += chunkSize) {
                chunks.push(arr.slice(i, i + chunkSize));
            }
            return chunks;
        };

        const batches = chunkArray(all, 25);

        for (const batch of batches) {
            console.log("Rewriting SNS");
            await this.actor.store_new_icrc1_canisters(batch);

        }
    }

    async addToCSV() {
        let canisters = await this.actor.count_icrc1_canisters() as number;
        const offset = 25
        let amountOfRequests = Math.ceil(Number(canisters) / offset);
        const canisters_from_oracle = await Promise.all(Array.from({length: amountOfRequests}, (_, i) =>
            this.actor.get_icrc1_paginated(i * offset, offset)
        )).then((res) => res.flat()) as Array<ICRC1>;

        let updatedMetadata = new Map<string, any>();
        for (const c of canisters_from_oracle) {
            try {
                const metadata = await getMetadata(c.ledger);
                updatedMetadata.set(c.ledger, metadata);
            } catch (e) {
                console.error(`Error while fetching metadata for ledger ${c.ledger} and name ${c.name}: ${e}`);
            }
        }

        const fields = ['name', 'symbol', 'ledger', 'index', 'category', 'logo', 'fee', 'decimals', 'root_canister_id', 'date_added'];
        const opts = {fields};
        try {
            const csv = (parse(await Promise.all(canisters_from_oracle
                    .map(async (c) => {
                        //update metadata. sometimes logo can be replaced
                        if (updatedMetadata.has(c.ledger)) {
                            let metadata = updatedMetadata.get(c.ledger);
                            c.name = metadata.name;
                            c.symbol = metadata.symbol;
                            c.logo = metadata.logo ? [metadata.logo] : c.logo;
                            c.decimals = metadata.decimals;
                            c.fee = metadata.fee;
                        }
                        let logo;
                        if (c.logo.length > 0) {
                            try {
                                logo = await compressLogo(c.logo[0])
                            } catch (e) {
                                console.error(`Error while compressing logo for ledger ${c.ledger} and name ${c.name}: ${e}`);
                                logo = c.logo[0]
                            }
                        } else {
                            logo = undefined
                        }

                        return {
                            name: c.name,
                            ledger: c.ledger,
                            category: mapCategory(c.category).toString(),
                            index: c.index.length > 0 ? c.index[0] : undefined,
                            symbol: c.symbol,
                            logo,
                            fee: c.fee.toString(),
                            decimals: c.decimals.toString(),
                            root_canister_id: c.root_canister_id.length > 0 ? c.root_canister_id[0] : undefined,
                            date_added: c.date_added.toString()
                        }
                    }))
                , opts));
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
                    index: record.index === undefined || record.index.length < 2 ? [] : [record.index],
                    symbol: record.symbol,
                    logo: record.logo === undefined ? [] : [record.logo],
                    fee: BigInt(record.fee),
                    decimals: Number(record.decimals),
                    root_canister_id: [record.root_canister_id],
                    date_added: BigInt(record.date_added)
                }
            });

        const chunkArray = (arr: ICRC1[], chunkSize: number): ICRC1[][] => {
            const chunks: ICRC1[][] = [];
            for (let i = 0; i < arr.length; i += chunkSize) {
                chunks.push(arr.slice(i, i + chunkSize));
            }
            return chunks;
        };

        const batches = chunkArray(asd, 10);

        for (const batch of batches) {
            console.log("Выгружаю CSV");
            await this.actor.replace_icrc1_canisters(batch);
        }
    }

    removeCanister(ledgerId: string) {
        return this.actor.remove_icrc1_canister(ledgerId);
    }

    getPrincipal(): string {
        console.log(this.adminED.getPrincipal().toText())
        return this.adminED.getPrincipal().toText()
    }

    async sendByList() {
        const csvData = fs.readFileSync(LIST_TO_SEND_PATH, 'utf8');
        const records: PairToSend[] = csvParse(csvData, {
            columns: true,
            skip_empty_lines: true,
        });

        let sender = this.adminED;

        for (const p of records) {
            stakeNeuron({
                identity: sender,
                canisterId: Principal.fromText(LEDGER_TO_SEND),
                stake: BigInt(p.tokens),
                controller: Principal.fromText(p.principal_id)
            })
        }
    }
}

export async function compressLogo(base64Logo: string): Promise<string> {
    const uri = base64Logo.split(';base64,')
    if (uri.length < 2 || uri[0] === "data:image/svg+xml" || uri[0] === "data:image/gif") {
        return base64Logo
    }
    const buffer = Buffer.from(uri.pop(), 'base64');
    if (buffer.length === 0) {
        return base64Logo
    }
    const resizedBuffer = await sharp(buffer)
        .resize(100, 100)
        .toBuffer();
    let format = uri.pop()
    return format + ";base64," + resizedBuffer.toString('base64');
}