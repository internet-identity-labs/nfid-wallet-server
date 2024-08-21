import {Dfx} from "./type/dfx";
import {deploy} from "./util/deployment.util";
import {App} from "./constanst/app.enum";
import {DFX} from "./constanst/dfx.const";
import {expect} from "chai";
import {ICRC1} from "./idl/icrc1_registry";

describe("ICRC1 canister Storage", () => {
    var dfx: Dfx;

    before(async () => {
        dfx = await deploy({apps: [App.ICRC1Registry]});
    });

    after(() => {
        DFX.STOP();
    });

    let canister_id = "id1";
    let one_more_canister_id = "id2";

    it("Store/retrieve canister id", async function () {
        await dfx.icrc1.actor.store_icrc1_canister(canister_id, { 'Active': null });
        await dfx.icrc1.actor.store_icrc1_canister(one_more_canister_id, { 'Inactive': null });
        let canisters = await dfx.icrc1.actor.get_canisters_by_root(dfx.user.identity.getPrincipal().toText()) as ICRC1[];
        expect(canisters.length).eq(2);
        expect(canisters[0].ledger).eq(canister_id);
        expect(canisters[0].state).deep.eq({ 'Active': null });
        expect(canisters[1].ledger).eq(one_more_canister_id);
        expect(canisters[1].state).deep.eq({ 'Inactive': null });
        await dfx.icrc1.actor.store_icrc1_canister(canister_id, { 'Inactive': null });
        canisters = await dfx.icrc1.actor.get_canisters_by_root(dfx.user.identity.getPrincipal().toText()) as ICRC1[];
        expect(canisters.find((c) => c.ledger === canister_id).state).deep.eq({ 'Inactive': null });
    })

    it("Remove canister", async function () {
        await dfx.icrc1.actor.remove_icrc1_canister(canister_id);
        let canisters = await dfx.icrc1.actor.get_canisters_by_root(dfx.user.identity.getPrincipal().toText()) as ICRC1[];
        expect(canisters.length).eq(1);
        expect(canisters[0].ledger).eq(one_more_canister_id);
    })
})