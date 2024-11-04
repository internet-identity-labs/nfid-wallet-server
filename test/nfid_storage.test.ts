import {Dfx} from "./type/dfx";
import {deploy, getActor, getIdentity} from "./util/deployment.util";
import {App} from "./constanst/app.enum";
import {expect} from "chai";
import {DFX} from "./constanst/dfx.const";
import {AccessPointRequest, HTTPAccountRequest, HTTPAccountResponse} from "./idl/identity_manager";
import {idlFactory as imIdl} from "./idl/identity_manager_idl";
import {idlFactory as nfidSIDL} from "./idl/nfid_storage_idl";
import {execute} from "./util/call.util";
import {PassKeyData} from "./idl/nfid_storage";

describe("NFID Storage test", () => {
    var dfx: Dfx;
    var storageActor: any;

    before(async () => {
        dfx = await deploy({apps: [App.IdentityManager, App.NFIDStorage]});
    });

    it("Store/Get passkey", async function () {
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

        storageActor = await getActor(dfx.nfid_storage.id, identity, nfidSIDL);

        let anchor = await storageActor.store_passkey("PASSKEY_ID", "SOME+TEST_STRING")

        expect(anchor).eq(100000000n)

        let passkey = await storageActor.get_passkey(["PASSKEY_ID"]) as PassKeyData

        expect(passkey[0].data).eq("SOME+TEST_STRING")

        execute(`dfx deploy nfid_storage  --argument '(opt record { im_canister = principal "${dfx.im.id}" })' --upgrade-unchanged`)

        passkey = await storageActor.get_passkey(["PASSKEY_ID"])

        expect(passkey[0].data).eq("SOME+TEST_STRING")
    })
})