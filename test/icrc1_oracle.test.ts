import {Dfx} from "./type/dfx";
import {deploy} from "./util/deployment.util";
import {App} from "./constanst/app.enum";
import {DFX} from "./constanst/dfx.const";
import {expect} from "chai";
import {ICRC1} from "./idl/icrc1_oracle";

describe("ICRC1 canister Oracle", () => {
    var dfx: Dfx;

    before(async () => {
        dfx = await deploy({apps: [App.ICRC1Oracle]});
    });

    after(() => {
        DFX.STOP();
    });

    it("Store/retrieve canister id", async function () {
        await dfx.icrc1_oracle.actor.sync_controllers();
        let firstCanister: ICRC1 = {
            logo: ["logo"],
            name: "name",
            ledger: "ryjl3-tyaaa-aaaaa-aaaba-cai",
            index: ["irshc-3aaaa-aaaam-absla-cai"],
            symbol: "symbol",
            category: {Spam: null},
            fee: BigInt(1),
            decimals: 1
        }
        await dfx.icrc1_oracle.actor.store_icrc1_canister(firstCanister);
        let allCanisters = await dfx.icrc1_oracle.actor.get_all_icrc1_canisters() as Array<ICRC1>;
        expect(allCanisters.length).eq(1);
        expect(allCanisters[0].ledger).eq("ryjl3-tyaaa-aaaaa-aaaba-cai");
        expect(allCanisters[0].name).eq("name");
        expect(allCanisters[0].symbol).eq("symbol");
        expect(allCanisters[0].index).deep.eq(["irshc-3aaaa-aaaam-absla-cai"]);
        expect(allCanisters[0].logo).deep.eq(["logo"]);
        expect(allCanisters[0].category).deep.eq({Community: null});

        const secondCanister: ICRC1 = {
            logo: ["logo2"],
            name: "name2",
            ledger: "irshc-3aaaa-aaaam-absla-cai",
            index: ["ryjl3-tyaaa-aaaaa-aaaba-cai"],
            symbol: "symbol2",
            category: {Spam: null},
            fee: BigInt(1),
            decimals: 1
        }
        const third: ICRC1 = {
            logo: ["logo3"],
            name: "name3",
            ledger: "c543j-2qaaa-aaaal-ac4dq-cai",
            index: ["ryjl3-tyaaa-aaaaa-aaaba-cai"],
            symbol: "symbol3",
            category: {Spam: null},
            fee: BigInt(1),
            decimals: 1
        }
        firstCanister = allCanisters[0]
        firstCanister.category = {Known: null}
        await dfx.icrc1_oracle.actor.replace_icrc1_canisters([firstCanister, secondCanister, third]);
        allCanisters = await dfx.icrc1_oracle.actor.get_all_icrc1_canisters() as Array<ICRC1>;
        expect(allCanisters.length).eq(3);
        expect(allCanisters.find((k) => k.ledger === firstCanister.ledger).category).deep.eq({Known: null});
    })
})