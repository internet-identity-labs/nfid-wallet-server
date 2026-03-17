import { idlFactory as icrcOracle1Idl } from "../test/idl/icrc1_oracle_idl";
import { Ed25519KeyIdentity } from "@dfinity/identity";
import { SnsParser } from "./sns";
import { NativeParser } from "./native";
import { ChainFusionParser } from "./chain_fusion";
import { ActorMethod } from "@dfinity/agent";
import { ICRC1 } from "../test/idl/icrc1_oracle";
import { parse } from "json2csv";
import * as fs from "fs";
import { parse as csvParse } from "csv-parse/sync";
import { ICRC1CsvData, DiscoveryAppCsvData } from "./types";
import { getActor, mapCategory, mapCategoryCSVToCategory } from "./util";
import { ChainFusionTestnetParser } from "./chain_fusion_testnet";
import { CANISTER_ID, FILE_PATH, FILE_PATH_NEURON, FILE_PATH_DISCOVERY, KEY_PAIR } from "./constants";
import { DiscoveryApp } from "../test/idl/icrc1_oracle";
import { DiscoveryStatus, DiscoveryApp as LocalDiscoveryApp } from "./discovery/types";
import { discoveryService } from "./discovery/discovery.service";
import { getMetadata } from "./metadata_service";
import sharp from "sharp";

export class AdminManager {
    private actor: Record<string, ActorMethod>;

    constructor() {
        const adminEd = Ed25519KeyIdentity.fromParsedJson(KEY_PAIR);
        this.actor = getActor(CANISTER_ID, adminEd, icrcOracle1Idl);
    }

    async addICTokens() {
        const sns = await new SnsParser().parseCanister();
        const native = await new NativeParser().parseCanister();
        const chainFusion = await new ChainFusionParser().parseCanister();
        const chainFusionTestnet = await new ChainFusionTestnetParser().parseCanister();
        const all = native.concat(chainFusion).concat(chainFusionTestnet).concat(sns);

        const chunkArray = (arr: ICRC1[], chunkSize: number): ICRC1[][] => {
            const chunks: ICRC1[][] = [];
            for (let i = 0; i < arr.length; i += chunkSize) {
                chunks.push(arr.slice(i, i + chunkSize));
            }
            return chunks;
        };

        const batches = chunkArray(all, 25);

        for (const batch of batches) {
            console.log("Importing and updating tokens metadata");
            await this.actor.store_new_icrc1_canisters(batch);
        }
    }

    async addToCSV() {
        let canisters = (await this.actor.count_icrc1_canisters()) as number;
        const offset = 25;
        let amountOfRequests = Math.ceil(Number(canisters) / offset);
        const canisters_from_oracle = (await Promise.all(
            Array.from({ length: amountOfRequests }, (_, i) => this.actor.get_icrc1_paginated(i * offset, offset))
        ).then((res) => res.flat())) as Array<ICRC1>;

        let updatedMetadata = new Map<string, any>();
        for (const c of canisters_from_oracle) {
            try {
                const metadata = await getMetadata(c.ledger);
                updatedMetadata.set(c.ledger, metadata);
            } catch (e) {
                console.error(`Error while fetching metadata for ledger ${c.ledger} and name ${c.name}: ${e}`);
            }
        }

        const fields = [
            "name",
            "symbol",
            "ledger",
            "index",
            "category",
            "logo",
            "fee",
            "decimals",
            "root_canister_id",
            "date_added",
        ];
        const opts = { fields };
        try {
            const csv = parse(
                await Promise.all(
                    canisters_from_oracle.map(async (c) => {
                        if (updatedMetadata.has(c.ledger)) {
                            let metadata = updatedMetadata.get(c.ledger);
                            c.name = metadata.name;
                            c.symbol = metadata.symbol;
                            c.logo = metadata.logo ? [metadata.logo] : c.logo;
                            c.decimals = metadata.decimals;
                            c.fee = metadata.fee;
                        }
                        let logo: string | undefined;
                        if (c.logo.length > 0) {
                            try {
                                if (c.logo[0]) {
                                    logo = await compressLogo(c.logo[0]);
                                } else {
                                    logo = undefined;
                                }
                            } catch (e) {
                                console.error(
                                    `Error while compressing logo for ledger ${c.ledger} and name ${c.name}: ${e}`
                                );
                                logo = c.logo[0] || undefined;
                            }
                        } else {
                            logo = undefined;
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
                            date_added: c.date_added.toString(),
                        };
                    })
                ),
                opts
            );
            fs.writeFileSync(FILE_PATH, csv);
            console.log("Tokens file saved successfully!");
        } catch (err) {
            console.error(err);
        }
    }

    async addFromCSV() {
        const csvData = fs.readFileSync(FILE_PATH, "utf8");
        const records: ICRC1CsvData[] = csvParse(csvData, {
            columns: true,
            skip_empty_lines: true,
        });
        const icrc1Records: ICRC1[] = records.map((record) => {
            return {
                name: record.name,
                ledger: record.ledger,
                category: mapCategoryCSVToCategory(record.category),
                index: record.index === undefined || record.index.length < 2 ? [] : [record.index],
                symbol: record.symbol,
                logo: record.logo === undefined ? [] : [record.logo],
                fee: BigInt(record.fee),
                decimals: Number(record.decimals),
                root_canister_id: record.root_canister_id ? [record.root_canister_id] : [],
                date_added: BigInt(record.date_added),
            };
        });

        const chunkArray = (arr: ICRC1[], chunkSize: number): ICRC1[][] => {
            const chunks: ICRC1[][] = [];
            for (let i = 0; i < arr.length; i += chunkSize) {
                chunks.push(arr.slice(i, i + chunkSize));
            }
            return chunks;
        };

        const batches = chunkArray(icrc1Records, 10);

        for (const batch of batches) {
            console.log("Exporting tokens metadata");
            await this.actor.replace_icrc1_canisters(batch);
        }
    }

    removeCanister(ledgerId: string) {
        return this.actor.remove_icrc1_canister(ledgerId);
    }

    async formNeuronsCSV() {
        const neurons = (await this.actor.get_all_neurons()) as Array<{
            name: string;
            date_added: bigint;
            ledger: string;
            neuron_id: string;
        }>;
        const fields = ["name", "ledger", "neuron_id", "date_added"];
        const opts = { fields };
        try {
            const csv = parse(neurons, opts);
            fs.writeFileSync(FILE_PATH_NEURON, csv);
            console.log("Neurons file saved successfully!");
        } catch (err) {
            console.error(err);
        }
    }

    async replaceNeuronsFromCSV() {
        const csvData = fs.readFileSync(FILE_PATH_NEURON, "utf8");
        const records: Array<{ name: string; ledger: string; neuron_id: string; date_added: string }> = csvParse(
            csvData,
            {
                columns: true,
                skip_empty_lines: true,
            }
        );
        const neuronsRecords = records.map((record) => {
            return {
                name: record.name,
                ledger: record.ledger,
                neuron_id: record.neuron_id,
                date_added: BigInt(Date.now()),
            };
        });
        await this.actor.replace_all_neurons(neuronsRecords);
    }

    async formDiscoveryCSV() {
        const offset = 50;
        let page = 0;
        const all: DiscoveryApp[] = [];
        while (true) {
            const batch = (await this.actor.get_discovery_app_paginated(BigInt(page * offset), BigInt(offset))) as DiscoveryApp[];
            all.push(...batch);
            if (batch.length < offset) break;
            page++;
        }
        const fields = ["id", "derivation_origin", "hostname", "url", "name", "icon", "desc", "is_global", "is_anonymous", "unique_users", "status"];
        const opts = { fields };
        try {
            const csv = parse(
                all.map((app) => ({
                    id: app.id.toString(),
                    derivation_origin: app.derivation_origin.length > 0 ? app.derivation_origin[0] : undefined,
                    hostname: app.hostname,
                    url: app.url.length > 0 ? app.url[0] : undefined,
                    name: app.name.length > 0 ? app.name[0] : undefined,
                    icon: app.icon.length > 0 ? app.icon[0] : undefined,
                    desc: app.desc.length > 0 ? app.desc[0] : undefined,
                    is_global: app.is_global.toString(),
                    is_anonymous: app.is_anonymous.toString(),
                    unique_users: app.unique_users.toString(),
                    status: Object.keys(app.status)[0],
                })),
                opts
            );
            fs.writeFileSync(FILE_PATH_DISCOVERY, csv);
            console.log("Discovery apps file saved successfully!");
        } catch (err) {
            console.error(err);
        }
    }

    async replaceDiscoveryFromCSV() {
        const csvData = fs.readFileSync(FILE_PATH_DISCOVERY, "utf8");
        const records: DiscoveryAppCsvData[] = csvParse(csvData, {
            columns: true,
            skip_empty_lines: true,
        });
        const apps: DiscoveryApp[] = records.map((record) => ({
            id: Number(record.id),
            derivation_origin: record.derivation_origin ? [record.derivation_origin] : [],
            hostname: record.hostname,
            url: record.url ? [record.url] : [],
            name: record.name ? [record.name] : [],
            icon: record.icon ? [record.icon] : [],
            desc: record.desc ? [record.desc] : [],
            is_global: record.is_global === "true",
            is_anonymous: record.is_anonymous === "true",
            unique_users: BigInt(record.unique_users),
            status: csvStatusToCandid(record.status),
        }));

        const chunkArray = (arr: DiscoveryApp[], chunkSize: number): DiscoveryApp[][] => {
            const chunks: DiscoveryApp[][] = [];
            for (let i = 0; i < arr.length; i += chunkSize) {
                chunks.push(arr.slice(i, i + chunkSize));
            }
            return chunks;
        };

        console.log("Clearing existing discovery apps");
        await this.actor.clear_discovery_apps();

        const batches = chunkArray(apps, 25);
        for (const batch of batches) {
            console.log("Uploading discovery apps batch");
            await this.actor.replace_all_discovery_app(batch);
        }
    }

    async enrichNewDiscoveryApps() {
        const csvData = fs.readFileSync(FILE_PATH_DISCOVERY, "utf8");
        const records: DiscoveryAppCsvData[] = csvParse(csvData, {
            columns: true,
            skip_empty_lines: true,
        });

        const newRecords = records.filter((r) => r.status === DiscoveryStatus.New);
        if (newRecords.length === 0) {
            console.log("No apps with status=New found.");
            return;
        }

        console.log(`Enriching ${newRecords.length} app(s) with status=New...`);

        const localApps: LocalDiscoveryApp[] = newRecords.map((r) => ({
            id: Number(r.id),
            derivationOrigin: r.derivation_origin,
            hostname: r.hostname,
            url: r.url,
            name: r.name,
            icon: r.icon,
            desc: r.desc,
            isGlobal: r.is_global === "true",
            isAnonymous: r.is_anonymous === "true",
            uniqueUsers: Number(r.unique_users),
            status: (r.status as DiscoveryStatus) ?? DiscoveryStatus.New,
        }));

        const responses = await discoveryService.getApps(localApps);

        // Merge enriched data back into the full records list
        const enrichedById = new Map<number, DiscoveryAppCsvData>();
        for (const resp of responses) {
            if (resp.isError) {
                console.error(`Failed to enrich app: ${resp.error.error}`);
                continue;
            }
            const app = resp.data;
            enrichedById.set(app.id, {
                id: app.id.toString(),
                derivation_origin: app.derivationOrigin,
                hostname: app.hostname,
                url: app.url,
                name: app.name,
                icon: app.icon,
                desc: app.desc,
                is_global: app.isGlobal.toString(),
                is_anonymous: app.isAnonymous.toString(),
                unique_users: app.uniqueUsers.toString(),
                status: DiscoveryStatus.Updated,
            });
        }

        if (enrichedById.size === 0) {
            console.log("No apps were successfully enriched.");
            return;
        }

        const updatedRecords = records.map((r) =>
            enrichedById.has(Number(r.id)) ? enrichedById.get(Number(r.id))! : r
        );

        const fields = ["id", "derivation_origin", "hostname", "url", "name", "icon", "desc", "is_global", "is_anonymous", "unique_users", "status"];
        const csv = parse(updatedRecords, { fields });
        fs.writeFileSync(FILE_PATH_DISCOVERY, csv);
        console.log(`Enriched ${enrichedById.size} app(s) and saved to ${FILE_PATH_DISCOVERY}. Run uploadDiscoveryCSV to push to canister.`);
    }
}

function csvStatusToCandid(status: string | undefined): { New: null } | { Updated: null } | { Verified: null } | { Spam: null } {
    switch (status) {
        case DiscoveryStatus.Updated:  return { Updated: null };
        case DiscoveryStatus.Verified: return { Verified: null };
        case DiscoveryStatus.Spam:     return { Spam: null };
        default:                       return { New: null };
    }
}

export async function compressLogo(base64Logo: string): Promise<string> {
    const uri = base64Logo.split(";base64,");
    if (uri.length < 2 || uri[0] === "data:image/svg+xml" || uri[0] === "data:image/gif") {
        return base64Logo;
    }
    if (uri.length === 0) {
        throw new Error("URI array is empty.");
    }
    const buffer = Buffer.from(uri.pop() as string, "base64");
    if (buffer.length === 0) {
        return base64Logo;
    }
    const resizedBuffer = await sharp(buffer).resize(100, 100).toBuffer();
    let format = uri.pop();
    return format + ";base64," + resizedBuffer.toString("base64");
}
