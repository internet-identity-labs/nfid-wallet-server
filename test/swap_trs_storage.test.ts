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
import {hasOwnProperty} from "../admin_oracle/util";

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
            name: [],
                challenge_attempt: [],
        };
        const actor = await getActor(dfx.im.id, identity, imIdl);
        await dfx.im.actor.add_email_and_principal_for_create_account_validation("test@test.test", principal, 25n);
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
            source_amount: 0n,
            swap_provider: {
                IcpSwap: null
            }
        }

        await storageActor.store_transaction(trs)

        let trss: SwapTransaction[] = await storageActor.get_transactions(identity.getPrincipal().toText())

        expect(trss.length).eq(1)

        expect(trss[0].uid).eq("123")
        expect(trss[0].withdraw[0]).eq(1n)
        expect(hasOwnProperty(trss[0].swap_provider, "IcpSwap")).is.true

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

    it("Store/Get notes", async function () {
        const identity = getIdentity("87654321876543218765432187654311");
        storageActor = await getTypedActor<_SERVICE>(dfx.swap_trs_storage.id, identity, swapStorageIDL);

        // bytes32 key: 32 bytes computed from chainId + transactionId
        const key = new Uint8Array(32).fill(0);
        key[0] = 1;
        key[31] = 255;

        const noteText = "Test note for transaction";

        const key2 = new Uint8Array(32).fill(2);
        const noteText2 = "Second note";

        await storageActor.store_note(key, noteText);
        await storageActor.store_note(key2, noteText2);

        // Query both keys at once
        const results = await storageActor.get_notes([key, key2]);
        expect(results.length).eq(2);
        const found1 = results.find(e => e.value === noteText);
        const found2 = results.find(e => e.value === noteText2);
        expect(found1).to.exist;
        expect(found2).to.exist;

        // Missing key returns no entry
        const unknownKey = new Uint8Array(32).fill(42);
        const missing = await storageActor.get_notes([unknownKey]);
        expect(missing.length).eq(0);

        // Verify notes survive upgrade
        execute(`dfx deploy swap_trs_storage  --argument '(opt record { im_canister = principal "${dfx.im.id}" })' --upgrade-unchanged`)
        const notesAfterUpgrade = await storageActor.get_notes([key, key2]);
        expect(notesAfterUpgrade.length).eq(2);
    })
})