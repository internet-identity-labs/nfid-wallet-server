import chai from "chai";
import chaiHttp from "chai-http";
import {ICRC1} from "../idl/icrc1_oracle";
import {RootCanister} from "./types";

chai.use(chaiHttp);


export class SnsParser {

    async parseCanister() {
        const response = await chai.request("https://icrc-api.internetcomputer.org")
            .get("/api/v1/ledgers?token_types=sns")
        const data = response.body.data as RootCanister[];
        let canisters: ICRC1[] = data.map((c) => {
            return {
                name: c.icrc1_metadata.icrc1_name,
                logo: c.icrc1_metadata.icrc1_logo ? [c.icrc1_metadata.icrc1_logo] : [],
                ledger: c.ledger_canister_id,
                index: [],
                symbol: c.icrc1_metadata.icrc1_symbol,
                category: {Sns: null}
            }
        });
        return canisters;
    }

}