// Copies the encrypted global keys from signer_ic (`get_all_json`) into ecdsa_storage.
//
//   node src/ecdsa_storage/scripts/import-global-keys.mjs --network ic --identity <controller of both> \
//        --source <signer_ic id> --canister <ecdsa_storage id> [--batch 400]
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
console.log(`${args.source} stores ${total} keys`);
const workDir = mkdtempSync(join(tmpdir(), "import-global-keys-"));
let imported = 0;
const skipped = [];
try {
    for (let from = 0; from < total; from += batchSize) {
        const to = Math.min(from + batchSize, total);
        // `get_all_json` returns JSON text, which dfx wraps into a JSON string.
        const records = JSON.parse(
            JSON.parse(dfxJson(args.source, "get_all_json", `(${from} : nat32, ${to} : nat32)`, ["--query"]))
        );
        const entries = [];
        for (const record of records) {
            // e.g. an ETH key (0x04...) stored in the IC signer by mistake; the lambda cannot sign with it either.
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
        if (entries.length > 0) {
            const argumentFile = join(workDir, `batch-${from}.did`);
            writeFileSync(argumentFile, `(vec { ${entries.join("; ")} })`);
            const result = JSON.parse(
                dfxJson(args.canister, "import_global_keys", null, ["--argument-file", argumentFile])
            );
            if ("Err" in result) throw new Error(`import_global_keys: ${result.Err}`);
            imported += entries.length;
            console.log(`Processed ${to}/${total}: imported ${imported}, canister stores ${result.Ok}`);
        }
    }
} finally {
    rmSync(workDir, { recursive: true, force: true });
}
console.log(`Done: imported ${imported}, skipped ${skipped.length}`);
for (const line of skipped) console.log(`  skipped ${line}`);

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
            maxBuffer: 64 * 1024 * 1024,
            env: { ...process.env, DFX_WARNING: "-mainnet_plaintext_identity" },
        }
    );
}
