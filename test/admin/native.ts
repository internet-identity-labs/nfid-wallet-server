import {ICRC1} from "../idl/icrc1_oracle";

export class NativeParser {

    async parseCanister(): Promise<ICRC1[]> {
        return [
            {
                name: "Internet Computer",
                logo: [],
                ledger: "ryjl3-tyaaa-aaaaa-aaaba-cai",
                index: ["qhbym-qaaaa-aaaaa-aaafq-cai"],
                symbol: "ICP",
                category: {Native: null}
            }
        ];
    }

}