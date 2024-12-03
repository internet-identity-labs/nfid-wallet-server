import chai from "chai";
import chaiHttp from "chai-http";
import {ICRC1} from "../idl/icrc1_oracle";
import {CanisterObject} from "./types";

chai.use(chaiHttp);


export class SnsParser {

    async parseCanister(): Promise<ICRC1[]> {
        let page = 0;
        let isRequestNeeded = true
        let data = []
        while (isRequestNeeded) {
            const response = await chai.request(`https://3r4gx-wqaaa-aaaaq-aaaia-cai.icp0.io/v1/sns/list/page/${page}/slow.json`
            ).get("")

            const generalCanisterInfos = response.body as any;
            page = page + 1
            if (generalCanisterInfos.length === 0) {
                isRequestNeeded = false
            }
            data = data.concat(generalCanisterInfos)
        }
        const canisterPromises = data.map((l) => {
            if (l.derived_state.sns_tokens_per_icp === 0) {
                return undefined
            }
            return chai.request(`https://sns-api.internetcomputer.org`)
                .get(`/api/v1/snses/${l.canister_ids.root_canister_id}`).then((a) => {
                    const c = a.body as CanisterObject;
                    if (c.enabled === false) {
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
                    return can
                });
        });
        const canisters = await Promise.all(canisterPromises.filter((c) => c !== undefined)) as ICRC1[];
        return canisters
    }
}