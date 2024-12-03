import {ICRC1} from "../idl/icrc1_oracle";
import {ICP_LOGO} from "./constants";

export class NativeParser {

    async parseCanister(): Promise<ICRC1[]> {
        return [
            {
                name: "Internet Computer",
                logo: [ICP_LOGO],
                ledger: "ryjl3-tyaaa-aaaaa-aaaba-cai",
                index: ["qhbym-qaaaa-aaaaa-aaafq-cai"],
                symbol: "ICP",
                category: {Native: null},
                fee: BigInt(10000),
                decimals: 8,
                root_canister_id: [],
                date_added: BigInt(Date.now())
            }
        ];
    }

}