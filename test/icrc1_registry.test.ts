import {Dfx} from "./type/dfx";
import {deploy} from "./util/deployment.util";
import {App} from "./constanst/app.enum";
import {DFX} from "./constanst/dfx.const";
import {expect} from "chai";

describe("ICRC1 canister Storage", () => {
    var dfx: Dfx;

    before(async () => {
        dfx = await deploy({apps: [App.ICRC1Registry]});
    });

    after(() => {
        DFX.STOP();
    });

    it("Store/retrieve canister id", async function () {
        let canister_id = "id1";
        let one_more_canister_id = "id2";
        await dfx.icrc1.actor.add_icrc1_canister(canister_id);
        await dfx.icrc1.actor.add_icrc1_canister(one_more_canister_id);
        let canisters = await dfx.icrc1.actor.get_canisters() as string[];
        expect(canisters.length).eq(2);
        expect(canisters[0]).eq(canister_id);
        expect(canisters[1]).eq(one_more_canister_id);
    })
})