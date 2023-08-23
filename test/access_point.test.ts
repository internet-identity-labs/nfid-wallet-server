import "mocha";
import {expect} from "chai";
import {Dfx} from "./type/dfx";
import {App} from "./constanst/app.enum";
import {deploy, getActor, getIdentity} from "./util/deployment.util";
import {
    AccessPointRemoveRequest,
    AccessPointRequest, HTTPAccessPointResponse,
    HTTPAccountRequest,
    HTTPAccountResponse,
} from "./idl/identity_manager";
import {DFX} from "./constanst/dfx.const";
import {idlFactory as imIdl} from "./idl/identity_manager_idl";
import {Ed25519KeyIdentity} from "@dfinity/identity";
import {fail} from "assert";

describe("Access Point", () => {

    var dfx: Dfx;

    before(async () => {
        dfx = await deploy({apps: [App.IdentityManager]});
    });

    after(() => {
        DFX.STOP();
    });

    it("should protect recovery phrase", async function () {
        const identity = getIdentity("87654321876543218765432187654311");
        const passKeyEmailRequest: AccessPointRequest = {
            icon: "Icon",
            device: "Global",
            pub_key: identity.getPrincipal().toText(),
            browser: "Browser",
            device_type: {
                Email: null
            },
            credential_id: []
        }
        var accountRequest: HTTPAccountRequest = {
            access_point: [passKeyEmailRequest],
            wallet: [{'NFID': null}],
            anchor: 0n,
            email: ["test@test.test"]
        };
        const actor = await getActor(dfx.im.id, identity, imIdl);
        const acc = (await actor.create_account(
            accountRequest
        )) as HTTPAccountResponse;
        expect(acc.status_code).eq(200)
        const recoveryIdentity = Ed25519KeyIdentity.generate();
        var request: AccessPointRequest = {
            icon: "",
            device: "",
            pub_key: recoveryIdentity.getPrincipal().toText(),
            browser: "",
            device_type: {
                Recovery: null
            },
            credential_id: []
        };
        let ap = (await actor.create_access_point(
            request
        ) )as HTTPAccessPointResponse
        expect(ap.status_code).eq(200)
        var recoveryRemoveRequest: AccessPointRemoveRequest = {
            pub_key: recoveryIdentity.getPrincipal().toText(),
        };
        try {
            await actor.remove_access_point(
                recoveryRemoveRequest
            )
            fail("")
        } catch (e) {
            expect(e.message).contains("Recovery phrase is protected")
        }
        let recoveryActor = await getActor(dfx.im.id, recoveryIdentity, imIdl);


        let pkIdentity = Ed25519KeyIdentity.generate()
        //get device back
        const passKeyRequest: AccessPointRequest = {
            icon: "Icon",
            device: "Global",
            pub_key: pkIdentity.getPrincipal().toText(),
            browser: "Browser",
            device_type: {
                Passkey: null
            },
            credential_id: []
        }
        ap = (await actor.create_access_point(
            passKeyRequest
        ) )as HTTPAccessPointResponse
        expect(ap.status_code).eq(200)

        //verify that recovery phrase does not affect pass keys
        let removeFromPKActor = await actor.remove_access_point(
            {
                pub_key: pkIdentity.getPrincipal().toText(),
            }
        ) as HTTPAccessPointResponse
        expect(removeFromPKActor.status_code).eq(200)

        //verify that you can remove recovery from recovery
        let resp = await recoveryActor.remove_access_point(
            recoveryRemoveRequest
        ) as HTTPAccessPointResponse
        expect(resp.status_code).eq(200)

        //verify that recovery principal removed from the index
        let resp2 = await recoveryActor.remove_access_point(
            {
                pub_key: identity.getPrincipal().toText(),
            }
        ) as HTTPAccessPointResponse
        expect(resp2.status_code).eq(404)

        //verify that we can remove root device (should not be a case for FE)
        let resp3 = await actor.remove_access_point(
            {
                pub_key: identity.getPrincipal().toText(),
            }
        ) as HTTPAccessPointResponse
        expect(resp3.status_code).eq(200)
    });
});