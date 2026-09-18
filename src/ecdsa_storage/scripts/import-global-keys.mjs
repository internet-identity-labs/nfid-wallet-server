// Copies the encrypted global keys from signer_ic (`get_all_json`) into ecdsa_storage.
//
//   node src/ecdsa_storage/scripts/import-global-keys.mjs --network ic --identity <controller of both> \
//        --source <signer_ic id> --canister <ecdsa_storage id> [--batch 400] [--from 0] [--dry-run]
//
// The identity must be an admin of signer_ic (a controller after its last sync_controllers) and a
// controller of ecdsa_storage. Records that cannot be stored (e.g. an ETH key written to the IC signer
// by mistake) are skipped and listed at the end. Interrupted runs resume with --from <index>.
import { execFileSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { parseArgs } from "node:util";

const { values: args } = parseArgs({
    options: {
        network: { type: "string", default: "ic" },
        identity: { type: "string" },
        source: { type: "string" },
        canister: { type: "string" },
        batch: { type: "string", default: "400" },
        from: { type: "string", default: "0" },
        "dry-run": { type: "boolean", default: false },
    },
});
if (!args.identity || !args.source || !args.canister) {
    throw new Error("--identity, --source and --canister are required");
}
const batchSize = Number(args.batch);
const HEX = /^[0-9a-fA-F]+$/;

// dfx prints nat64 as a JSON string.
const total = Number(JSON.parse(dfxJson(args.source, "count", "()", ["--query"])));
if (!Number.isSafeInteger(total) || total < 0) {
    throw new Error(`Unexpected key count from ${args.source}`);
}
const storedBefore = globalKeysInCanister();
console.log(
    `${args.source} stores ${total} keys; ${args.canister} stores ${storedBefore}${args["dry-run"] ? " (dry run)" : ""}`
);

const workDir = mkdtempSync(join(tmpdir(), "import-global-keys-"));
let imported = 0;
const skipped = [];
try {
    for (let from = Number(args.from); from < total; from += batchSize) {
        const to = Math.min(from + batchSize, total);
        // `get_all_json` returns JSON text, which dfx wraps into a JSON string.
        const records = JSON.parse(
            JSON.parse(dfxJson(args.source, "get_all_json", `(${from} : nat32, ${to} : nat32)`, ["--query"]))
        );
        const entries = [];
        for (const record of records) {
            const reason = !/^[a-z0-9-]+$/.test(record.principal)
                ? "invalid root"
                : !HEX.test(record.public_key) || !HEX.test(record.private_key)
                  ? "not a hex-encoded key pair"
                  : null;
            if (reason) {
                skipped.push(`${record.principal}: ${reason}`);
                continue;
            }
            entries.push(
                `record { root = "${record.principal}"; public_key = "${record.public_key}"; private_key_encrypted = "${record.private_key}" }`
            );
        }
        if (entries.length > 0 && !args["dry-run"]) {
            const argumentFile = join(workDir, `batch-${from}.did`);
            writeFileSync(argumentFile, `(vec { ${entries.join("; ")} })`);
            const result = JSON.parse(
                dfxJson(args.canister, "import_global_keys", null, ["--argument-file", argumentFile])
            );
            if ("Err" in result) throw new Error(`import_global_keys at ${from}: ${result.Err}`);
            imported += entries.length;
            console.log(`Read ${to}/${total}: imported ${imported}, canister stores ${result.Ok}`);
        } else {
            imported += entries.length;
            console.log(`Read ${to}/${total}: ${imported} importable${args["dry-run"] ? " (nothing sent)" : ""}`);
        }
    }
} catch (error) {
    console.error(`Failed after reading ${imported + skipped.length} records; resume with --from <last read index>`);
    throw error;
} finally {
    rmSync(workDir, { recursive: true, force: true });
}

console.log(`Done: ${args["dry-run"] ? "importable" : "imported"} ${imported}, skipped ${skipped.length}`);
for (const line of skipped) console.log(`  skipped ${line}`);
if (!args["dry-run"]) {
    const storedAfter = globalKeysInCanister();
    console.log(`${args.canister} stores ${storedAfter} keys (source ${total}, skipped ${skipped.length})`);
    if (Number(args.from) === 0 && storedAfter !== total - skipped.length) {
        throw new Error("Stored key count does not match the source");
    }
}

function globalKeysInCanister() {
    const status = JSON.parse(dfxJson(args.canister, "status", "()", ["--query"]));
    if ("Err" in status) throw new Error(`status: ${status.Err}`);
    return Number(status.Ok.global_keys);
}

function dfxJson(canister, method, argument, extra = []) {
    return execFileSync(
        "dfx",
        [
            "canister",
            "call",
            "--network",
            args.network,
            "--identity",
            args.identity,
            ...extra,
            "--output",
            "json",
            canister,
            method,
            ...(argument === null ? [] : [argument]),
        ],
        {
            encoding: "utf8",
            maxBuffer: 256 * 1024 * 1024,
            env: { ...process.env, DFX_WARNING: "-mainnet_plaintext_identity" },
        }
    );
}
