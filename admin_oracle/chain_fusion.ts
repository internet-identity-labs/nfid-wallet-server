import {ICRC1} from "../test/idl/icrc1_oracle";
import {RootCanister} from "./types";

export class ChainFusionParser {
    async parseCanister(): Promise<ICRC1[]> {
        const response = await fetch("https://icrc-api.internetcomputer.org/api/v1/ledgers?token_types=ckerc20_mainnet");
        const body = await response.json() as { data: RootCanister[] };
        return body.data.map((c) => ({
            name: c.icrc1_metadata.icrc1_name,
            logo: c.icrc1_metadata.icrc1_logo ? [c.icrc1_metadata.icrc1_logo] : [],
            ledger: c.ledger_canister_id,
            index: [],
            symbol: c.icrc1_metadata.icrc1_symbol,
            category: {ChainFusion: null},
            fee: BigInt(c.icrc1_metadata.icrc1_fee),
            decimals: parseInt(c.icrc1_metadata.icrc1_decimals),
            root_canister_id: c.sns_root_canister_id ? [c.sns_root_canister_id] : [],
            date_added: BigInt(Date.now())
        }));
    }
}
