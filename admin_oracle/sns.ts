import {ICRC1} from "../test/idl/icrc1_oracle";
import {CanisterObject} from "./types";

export class SnsParser {

    async parseCanister(): Promise<ICRC1[]> {
        let page = 0;
        let isRequestNeeded = true
        let data = []
        while (isRequestNeeded) {
            const url = `https://3r4gx-wqaaa-aaaaq-aaaia-cai.icp0.io/v1/sns/list/page/${page}/slow.json`;
            console.log(`[sns] fetching page ${page}: ${url}`);
            const response = await fetch(url);
            console.log(`[sns] page ${page} status: ${response.status}`);
            if (!response.ok) {
                console.log(`[sns] page ${page} non-OK status, stopping pagination`);
                isRequestNeeded = false;
                page = page + 1;
                continue;
            }
            const generalCanisterInfos = await response.json() as any;
            console.log(`[sns] page ${page} items: ${generalCanisterInfos?.length ?? 0}`);
            page = page + 1
            if (!Array.isArray(generalCanisterInfos) || generalCanisterInfos.length === 0) {
                isRequestNeeded = false
            } else {
                data = data.concat(generalCanisterInfos)
            }
        }
        console.log(`[sns] total SNS entries: ${data.length}, starting canister detail fetch`);
        const canisterPromises = data.map((l, i) => {
            if (l.derived_state.sns_tokens_per_icp === 0) {
                return undefined
            }
            const detailUrl = `https://sns-api.internetcomputer.org/api/v1/snses/${l.canister_ids.root_canister_id}`;
            console.log(`[sns] [${i}] fetching canister detail: ${l.canister_ids.root_canister_id}`);
            return fetch(detailUrl)
                .then((a) => {
                    console.log(`[sns] [${i}] canister detail status: ${a.status}`);
                    return a.json();
                })
                .then((c: CanisterObject) => {
                    if (c.enabled === false) {
                        console.log(`[sns] [${i}] canister disabled, skipping`);
                        return undefined
                    }
                    let can: ICRC1 = {
                        name: c.icrc1_metadata.icrc1_name,
                        logo: c.logo ? [c.logo] : [],
                        ledger: c.ledger_canister_id,
                        index: c.index_canister_id ? [c.index_canister_id] : [],
                        symbol: c.icrc1_metadata.icrc1_symbol,
                        category: {Sns: null},
                        fee: BigInt(c.icrc1_metadata.icrc1_fee),
                        decimals: parseInt(c.icrc1_metadata.icrc1_decimals),
                        root_canister_id: [l.canister_ids.root_canister_id],
                        date_added: BigInt(Date.now())
                    }
                    console.log(`[sns] [${i}] parsed: ${can.symbol} (${can.ledger})`);
                    return can
                })
                .catch((e) => {
                    console.error(`[sns] [${i}] failed to fetch ${l.canister_ids.root_canister_id}:`, e.message);
                    return undefined;
                });
        });
        console.log(`[sns] waiting for ${canisterPromises.filter(Boolean).length} canister detail requests`);
        const canisters = await Promise.all(canisterPromises.filter((c) => c !== undefined)) as ICRC1[];
        const result = canisters.filter((c) => c !== undefined) as ICRC1[];
        console.log(`[sns] done, total canisters: ${result.length}`);
        return result;
    }
}
