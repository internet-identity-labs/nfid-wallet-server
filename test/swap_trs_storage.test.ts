import {Dfx} from "./type/dfx";
import {deploy, getActor, getIdentity, getTypedActor} from "./util/deployment.util";
import {App} from "./constanst/app.enum";
import {expect} from "chai";
import {AccessPointRequest, HTTPAccountRequest, HTTPAccountResponse} from "./idl/identity_manager";
import {idlFactory as imIdl} from "./idl/identity_manager_idl";
import {idlFactory as swapStorageIDL} from "./idl/swap_trs_storage_idl";
import {_SERVICE, SwapTransaction} from "./idl/swap_trs_storage";
import {execute} from "./util/call.util";
import * as Agent from "@dfinity/agent";

describe("Swap Trs Storage test", () => {
    var dfx: Dfx;
    var storageActor: Agent.ActorSubclass<_SERVICE>;

    before(async () => {
        dfx = await deploy({apps: [App.IdentityManager, App.SwapTrsStorage]});
    });

    it("Store/Get transactions", async function () {
        const identity = getIdentity("87654321876543218765432187654311");
        const principal = identity.getPrincipal().toText();
        const dd: AccessPointRequest = {
            icon: "Icon",
            device: "Global",
            pub_key: principal,
            browser: "Browser",
            device_type: {
                Email: null,
            },
            credential_id: [],
        };
        var accountRequest: HTTPAccountRequest = {
            access_point: [dd],
            wallet: [{NFID: null}],
            anchor: 0n,
            email: ["test@test.test"],
        };
        const actor = await getActor(dfx.im.id, identity, imIdl);
        await dfx.im.actor.add_email_and_principal_for_create_account_validation("test@test.test", principal, 25);
        const accResponse: HTTPAccountResponse = (await actor.create_account(
            accountRequest
        )) as HTTPAccountResponse;
        const response = accResponse.data[0];
        expect(response.anchor).eq(100000000n);

        console.log(dfx.swap_trs_storage.id)

        storageActor = await getTypedActor<_SERVICE>(dfx.swap_trs_storage.id, identity, swapStorageIDL);

        const trs: SwapTransaction = {
            uid: "123",
            withdraw: [1n],
            swap: [2n],
            deposit: [3n],
            end_time: [4n],
            transfer_id: [5n],
            target_ledger: "Ledger",
            errors: [{ time: 0n, message: "Error" }],
            stage: {
                Completed: null
            },
            start_time: 0n,
            source_ledger: "Source",
            transfer_nfid_id: [0n],
            target_amount: 0n,
            source_amount: 0n
        }

        await storageActor.store_transaction(trs)

        let trss: SwapTransaction[] = await storageActor.get_transactions(identity.getPrincipal().toText())

        expect(trss.length).eq(1)

        expect(trss[0].uid).eq("123")
        expect(trss[0].withdraw[0]).eq(1n)

        trs.withdraw = [2n]

        await storageActor.store_transaction(trs)

        trss = await storageActor.get_transactions(identity.getPrincipal().toText())

        expect(trss.length).eq(1)
        expect(trss[0].withdraw[0]).eq(2n)

        execute(`dfx deploy swap_trs_storage  --argument '(opt record { im_canister = principal "${dfx.im.id}" })' --upgrade-unchanged`)

        let trsFromMemory = await storageActor.get_transactions(identity.getPrincipal().toText())
        expect(trsFromMemory.length).eq(1)
        expect(trsFromMemory[0].errors.length).eq(1)
    })
})