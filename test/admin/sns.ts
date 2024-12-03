import chai from "chai";
import chaiHttp from "chai-http";
import {ICRC1} from "../idl/icrc1_oracle";
import {CanisterObject, CanisterObjectA} from "./types";

chai.use(chaiHttp);


export class SnsParser {

    async parseCanister(): Promise<ICRC1[]> {
        const response = await chai.request("https://sns-api.internetcomputer.org")
            .get("/api/v1/snses")
        const data = response.body.data as CanisterObjectA[];
        const canisterPromises = data.map((l) => {
            return  chai.request(`https://sns-api.internetcomputer.org`)
                .get(`/api/v1/snses/${l.root_canister_id}`).then((a) => {
                const c = a.body as CanisterObject;
                return {
                    name: c.icrc1_metadata.icrc1_name,
                    logo: c.logo ? [c.logo] : [],
                    ledger: c.ledger_canister_id,
                    index: [],
                    symbol: c.icrc1_metadata.icrc1_symbol,
                    category: {Sns: null},
                    fee: BigInt(c.icrc1_metadata.icrc1_fee),
                    decimals: parseInt(c.icrc1_metadata.icrc1_decimals),
                    root_canister_id: [l.root_canister_id],
                    date_added: BigInt(Date.now())
                } as ICRC1
            });
        });
        const canisters = await Promise.all(canisterPromises);
        return canisters
    }
}