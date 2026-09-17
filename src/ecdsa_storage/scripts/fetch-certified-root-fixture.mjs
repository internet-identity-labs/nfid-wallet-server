// Fetches a real `get_root_certified` response from the Identity Manager for the certificate
// verification tests: node src/ecdsa_storage/scripts/fetch-certified-root-fixture.mjs [im_canister_id]
//
// Uses the legacy test access point from nfid-frontend (packages/integration/src/lib/lambda/ecdsa.spec.ts).
import { writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { Actor, HttpAgent } from "@dfinity/agent";
import { Ed25519KeyIdentity } from "@dfinity/identity";

const IM_CANISTER_ID = process.argv[2] ?? "74gpt-tiaaa-aaaak-aacaa-cai";
const TEST_ACCESS_POINT = [
    "302a300506032b65700321003b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29",
    "00000000000000000000000000000000000000000000000000000000000000003b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29",
];
const OUTPUT = fileURLToPath(new URL("../tests/fixtures/im_dev_certified_root_api_v2.json", import.meta.url));

const idlFactory = ({ IDL }) =>
    IDL.Service({
        get_root_certified: IDL.Func(
            [],
            [IDL.Record({ response: IDL.Text, certificate: IDL.Vec(IDL.Nat8), witness: IDL.Vec(IDL.Nat8) })],
            ["query"]
        ),
    });

const identity = Ed25519KeyIdentity.fromParsedJson(TEST_ACCESS_POINT);
const agent = await HttpAgent.create({ host: "https://icp-api.io", identity });
const actor = Actor.createActor(idlFactory, { agent, canisterId: IM_CANISTER_ID });
const response = await actor.get_root_certified();

const toHex = (bytes) => Buffer.from(bytes).toString("hex");
writeFileSync(
    OUTPUT,
    JSON.stringify(
        {
            im_canister: IM_CANISTER_ID,
            caller: identity.getPrincipal().toText(),
            root: response.response,
            certificate: toHex(response.certificate),
            witness: toHex(response.witness),
        },
        null,
        2
    ) + "\n"
);
console.log(`caller ${identity.getPrincipal().toText()} -> root ${response.response}; written to ${OUTPUT}`);
