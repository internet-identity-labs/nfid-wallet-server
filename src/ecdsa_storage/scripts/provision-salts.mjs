// Delivers the lambda salts to ecdsa_storage without exposing them in ingress messages.
//
//   node src/ecdsa_storage/scripts/provision-salts.mjs --network ic --identity <controller> --canister <id> \
//        [--serverless-template <.serverless/cloudformation-template-update-stack.json>]
//
// Salts come from the deployed lambda template when given, otherwise from ECDSA_SALT and ANONYMOUS_SALT.
// The scheme must match src/provisioning.rs.
import { execFileSync } from "node:child_process";
import crypto from "node:crypto";
import { readFileSync } from "node:fs";
import { parseArgs } from "node:util";

const KDF_DOMAIN = Buffer.from("nfid-ecdsa-storage/provision-salts/v1");
const FINGERPRINT_DOMAIN = Buffer.from("nfid-ecdsa-storage/salts-fingerprint/v1");
const X25519_SPKI_PREFIX = Buffer.from("302a300506032b656e032100", "hex");

const { values: args } = parseArgs({
    options: {
        network: { type: "string", default: "ic" },
        identity: { type: "string" },
        canister: { type: "string" },
        "serverless-template": { type: "string" },
    },
});
if (!args.identity || !args.canister) {
    throw new Error("--identity and --canister are required");
}

const salts = readSalts();
const recipient = Buffer.from(callOk("get_provisioning_key", "()"), "hex");
const sealed = seal(salts, recipient, principalBytes(args.canister));
callOk(
    "provision_salts",
    `(record { ephemeral_public_key = "${sealed.ephemeralPublicKey}"; nonce = "${sealed.nonce}"; ciphertext = "${sealed.ciphertext}" })`
);

const status = callOk("status", "()", ["--query"]);
const expected = fingerprint(salts);
if (status.salts_fingerprint?.[0] !== expected && status.salts_fingerprint !== expected) {
    throw new Error(`Fingerprint mismatch: canister ${JSON.stringify(status.salts_fingerprint)}, expected ${expected}`);
}
console.log(`Salts provisioned to ${args.canister}, fingerprint ${expected}`);

function readSalts() {
    if (args["serverless-template"]) {
        const resources = JSON.parse(readFileSync(args["serverless-template"], "utf8")).Resources;
        const variables = resources.EcdsaUnderscoregetUnderscoreanonymousLambdaFunction.Properties.Environment.Variables;
        return { ecdsa_salt: variables.ECDSA_SALT, anonymous_salt: variables.ANONYMOUS_SALT };
    }
    return { ecdsa_salt: process.env.ECDSA_SALT, anonymous_salt: process.env.ANONYMOUS_SALT };
}

function seal(payload, recipientPublicKey, aad) {
    if (!payload.ecdsa_salt || !payload.anonymous_salt) throw new Error("Both salts are required");
    const { publicKey, privateKey } = crypto.generateKeyPairSync("x25519");
    const ephemeralPublicKey = publicKey.export({ format: "der", type: "spki" }).subarray(-32);
    const shared = crypto.diffieHellman({
        privateKey,
        publicKey: crypto.createPublicKey({
            key: Buffer.concat([X25519_SPKI_PREFIX, recipientPublicKey]),
            format: "der",
            type: "spki",
        }),
    });
    const key = crypto
        .createHash("sha256")
        .update(Buffer.concat([KDF_DOMAIN, shared, ephemeralPublicKey, recipientPublicKey]))
        .digest();
    const nonce = crypto.randomBytes(12);
    const plaintext = Buffer.from(JSON.stringify(payload));
    const cipher = crypto.createCipheriv("chacha20-poly1305", key, nonce, { authTagLength: 16 });
    cipher.setAAD(aad, { plaintextLength: plaintext.length });
    const ciphertext = Buffer.concat([cipher.update(plaintext), cipher.final(), cipher.getAuthTag()]);
    return {
        ephemeralPublicKey: ephemeralPublicKey.toString("hex"),
        nonce: nonce.toString("hex"),
        ciphertext: ciphertext.toString("hex"),
    };
}

function fingerprint({ ecdsa_salt, anonymous_salt }) {
    const hash = crypto.createHash("sha256").update(FINGERPRINT_DOMAIN);
    for (const salt of [ecdsa_salt, anonymous_salt]) {
        const length = Buffer.alloc(8);
        length.writeBigUInt64BE(BigInt(Buffer.byteLength(salt)));
        hash.update(length).update(salt);
    }
    return hash.digest().subarray(0, 8).toString("hex");
}

function principalBytes(text) {
    const alphabet = "abcdefghijklmnopqrstuvwxyz234567";
    let bits = 0;
    let value = 0;
    const bytes = [];
    for (const char of text.replace(/-/g, "").toLowerCase()) {
        value = (value << 5) | alphabet.indexOf(char);
        bits += 5;
        if (bits >= 8) {
            bytes.push((value >>> (bits - 8)) & 0xff);
            bits -= 8;
        }
    }
    return Buffer.from(bytes.slice(4)); // drop the CRC32 prefix
}

function callOk(method, argument, extra = []) {
    const output = execFileSync(
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
            args.canister,
            method,
            argument,
        ],
        { encoding: "utf8", env: { ...process.env, DFX_WARNING: "-mainnet_plaintext_identity" } }
    );
    const result = JSON.parse(output);
    if ("Err" in result) throw new Error(`${method}: ${result.Err}`);
    return result.Ok;
}
