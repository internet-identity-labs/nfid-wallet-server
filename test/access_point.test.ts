import "mocha";
import {expect} from "chai";
import {Dfx} from "./type/dfx";
import {App} from "./constanst/app.enum";
import {deploy, getActor, getIdentity, getTypedActor} from "./util/deployment.util";
import {
    AccessPointRemoveRequest,
    AccessPointRequest, BoolHttpResponse, CertifiedResponse, HTTPAccessPointResponse,
    HTTPAccountRequest,
    HTTPAccountResponse,
} from "./idl/identity_manager";
import {DFX} from "./constanst/dfx.const";
import {idlFactory as imIdl} from "./idl/identity_manager_idl";
import {Ed25519KeyIdentity} from "@dfinity/identity";
import {fail} from "assert";
import { _SERVICE as IdentityManagerType } from "./idl/identity_manager"
import { verifyCertifiedResponse } from "./util/cert_verification";
import {call} from "./util/call.util";
import {buildIdentity, setupNfidAccount} from "./util/nfid_account.util";

describe("Access Point", () => {

    var dfx: Dfx;

    before(async () => {
        dfx = await deploy({apps: [App.IdentityManager]});
    });

    it("should protect recovery phrase", async function () {
        const identity = getIdentity("87654321876543218765432187654311");
        const principal = identity.getPrincipal().toText();

        console.log(call(`dfx canister call identity_manager get_config`))

        const passKeyEmailRequest: AccessPointRequest = {
            icon: "Icon",
            device: "Global",
            pub_key: principal,
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
            email: ["test@test.test"],
            name: [],
            challenge_attempt: []
        };

        let response = await dfx.im.actor.add_email_and_principal_for_create_account_validation("test@test.test", principal, 25n) as BoolHttpResponse;
        expect(response.status_code).eq(200);

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

        let recoveryActor = await getActor(dfx.im.id, recoveryIdentity, imIdl);
        //verify certified response for passkey
        var certifiedResponse = (await recoveryActor.get_root_certified())as CertifiedResponse
        expect(certifiedResponse.witness.length > 0).eq(true)
        expect(certifiedResponse.response).eq(identity.getPrincipal().toText())

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
        //verify certified response for recovery
        certifiedResponse = (await recoveryActor.get_root_certified())as CertifiedResponse
        expect(certifiedResponse.witness.length > 0).eq(true)
        expect(certifiedResponse.response).eq(identity.getPrincipal().toText())
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

        //verify certified response removed for recovery
        try{
            await recoveryActor.get_root_certified()
            fail("Nope")
        }catch (e) {
            expect(e.message).contains("No such ap")
        }

        //verify that recovery principal removed from the index
        let resp2 = await recoveryActor.remove_access_point(
            {
                pub_key: identity.getPrincipal().toText(),
            }
        ) as HTTPAccessPointResponse
        expect(resp2.status_code).eq(404)

        //cannot remove the email device once no passkey and recovery are left to fall back on
        let resp3 = await actor.remove_access_point(
            {
                pub_key: identity.getPrincipal().toText(),
            }
        ) as HTTPAccessPointResponse
        expect(resp3.status_code).eq(403)
        expect(resp3.error[0]).contains("passkey and recovery access points required")
    });

    it("should have device principal in certified map after app restarts.", async function () {
        const identity = getIdentity("87654321876543218765432187654377");
        const principal = identity.getPrincipal().toText();
        const email = "test@test.test";

        const validationResponse = await dfx.im.actor.add_email_and_principal_for_create_account_validation(email, principal, 25n);
        expect(validationResponse.status_code).eq(200);

        const accessPointRequest: AccessPointRequest = {
            icon: "google",
            device: "Google",
            pub_key: principal,
            browser: "",
            device_type: {
                Email: null
            },
            credential_id: []
        }

        var accountRequest: HTTPAccountRequest = {
            access_point: [accessPointRequest],
            wallet: [{ NFID: null }],
            anchor: 0n,
            email: [email],
            name: [],
                challenge_attempt: []
        };

        const actor = await getTypedActor<IdentityManagerType>(dfx.im.id, identity, imIdl);
        const accountResponse = await actor.create_account(accountRequest);
        expect(accountResponse.status_code).eq(200);

        var rootCertifiedResponse = await actor.get_root_certified();
        expect(rootCertifiedResponse.witness.length > 0).eq(true);
        expect(rootCertifiedResponse.response).eq(principal);

        // Add recovery device.

        const recoveryIdentity = Ed25519KeyIdentity.generate();
        const recoveryPrincipal = recoveryIdentity.getPrincipal().toText();
        var request: AccessPointRequest = {
            icon: "",
            device: "",
            pub_key: recoveryPrincipal,
            browser: "",
            device_type: {
                Recovery: null
            },
            credential_id: []
        };
        let accessPointResponse = await actor.create_access_point(request);
        expect(accessPointResponse.status_code).eq(200)

        // Check existence of the device in device index.

        let recoveryActor = await getTypedActor<IdentityManagerType>(dfx.im.id, recoveryIdentity, imIdl);
        var {certificate, witness, response} = await recoveryActor.get_root_certified();
        expect(response).eq(identity.getPrincipal().toText());
        await verifyCertifiedResponse(certificate, witness, recoveryPrincipal, dfx.im.id, response);

        // Restart the app.

        dfx = await deploy({clean: false, apps: [App.IdentityManager]});

        // It should have no device in device index after restart.
        
        var {certificate, witness, response} = await recoveryActor.get_root_certified();
        expect(response).eq(identity.getPrincipal().toText());

        try {
            await verifyCertifiedResponse(certificate, witness, recoveryPrincipal, dfx.im.id, response);
        } catch(e) {
            expect(e.message).to.equal("Tree root hash did not match the certified data in the certificate.");
        }

        // It should have it restored by the methods.

        const successStackResponse = await dfx.im.actor.save_temp_stack_to_rebuild_device_index();
        expect(successStackResponse).to.equal("The stack has been filled with data.");

        const errorStackResponse = await dfx.im.actor.save_temp_stack_to_rebuild_device_index();
        expect(errorStackResponse).to.equal("The stack is not empty. No action required.");

        const nonZeroRemainingAmount = await dfx.im.actor.get_remaining_size_after_rebuild_device_index_slice_from_temp_stack([1n]);
        expect(Number(nonZeroRemainingAmount)).to.be.gt(0);

        const zeroRemainingAmount = await dfx.im.actor.get_remaining_size_after_rebuild_device_index_slice_from_temp_stack([10000n]);
        expect(Number(zeroRemainingAmount)).to.equal(0);

        const zeroRemainingAmount2 = await dfx.im.actor.get_remaining_size_after_rebuild_device_index_slice_from_temp_stack([]);
        expect(Number(zeroRemainingAmount2)).to.equal(0);

        var {certificate, witness, response} = await recoveryActor.get_root_certified();
        expect(response).eq(identity.getPrincipal().toText());
        await verifyCertifiedResponse(certificate, witness, recoveryPrincipal, dfx.im.id, response);
    });

    describe("Remove email access point", () => {

        it("should clear the account email and remove the device when passkey and recovery remain", async function () {
            // Given an NFID account with an Email access point plus a passkey and a recovery device
            const {emailIdentity, emailActor, passkeyActor} = await setupNfidAccount(dfx, "clear-ok", {
                email: "ap-clear-ok@test.test",
                withPasskey: true,
                withRecovery: true
            });
            const emailPrincipal = emailIdentity.getPrincipal().toText();

            // When the Email access point is removed
            const removeResponse = await emailActor.remove_access_point({
                pub_key: emailPrincipal
            });

            // Then the removal succeeds and the account email is cleared
            expect(removeResponse.status_code).eq(200);
            expect(removeResponse.data[0].some(ap => ap.principal_id === emailPrincipal)).eq(false);

            // The email principal can no longer resolve the account, so read through the passkey.
            const account = await passkeyActor.get_account();
            expect(account.status_code).eq(200);
            expect(account.data[0].email).deep.eq([]);
            expect(account.data[0].access_points.some(ap => 'Email' in ap.device_type)).eq(false);
        });

        it("should reject removal with 403 when no passkey would remain", async function () {
            // Given an NFID account with an Email access point and a recovery device but no passkey
            const {emailIdentity, emailActor} = await setupNfidAccount(dfx, "no-passkey", {
                email: "ap-no-passkey@test.test",
                withPasskey: false,
                withRecovery: true
            });
            const emailPrincipal = emailIdentity.getPrincipal().toText();

            // When the Email access point is removed
            const removeResponse = await emailActor.remove_access_point({
                pub_key: emailPrincipal
            });

            // Then the removal is rejected and the account is unchanged
            expect(removeResponse.status_code).eq(403);
            expect(removeResponse.error[0]).contains("passkey and recovery access points required");

            const account = await emailActor.get_account();
            expect(account.status_code).eq(200);
            expect(account.data[0].email).deep.eq(["ap-no-passkey@test.test"]);
            expect(account.data[0].access_points.some(ap => 'Email' in ap.device_type)).eq(true);
        });

        it("should reject removal with 403 when no recovery would remain", async function () {
            // Given an NFID account with an Email access point and a passkey but no recovery device
            const {emailIdentity, emailActor} = await setupNfidAccount(dfx, "no-recovery", {
                email: "ap-no-recovery@test.test",
                withPasskey: true,
                withRecovery: false
            });
            const emailPrincipal = emailIdentity.getPrincipal().toText();

            // When the Email access point is removed
            const removeResponse = await emailActor.remove_access_point({
                pub_key: emailPrincipal
            });

            // Then the removal is rejected and the account is unchanged
            expect(removeResponse.status_code).eq(403);
            expect(removeResponse.error[0]).contains("passkey and recovery access points required");

            const account = await emailActor.get_account();
            expect(account.status_code).eq(200);
            expect(account.data[0].email).deep.eq(["ap-no-recovery@test.test"]);
            expect(account.data[0].access_points.some(ap => 'Email' in ap.device_type)).eq(true);
        });

        it("should leave the account email untouched when a non-email device is removed", async function () {
            // Given an NFID account with Email, passkey, and recovery access points
            const {emailActor, passkeyIdentity} = await setupNfidAccount(dfx, "keep-email", {
                email: "ap-keep-email@test.test",
                withPasskey: true,
                withRecovery: true
            });

            // When the passkey access point is removed
            const removeResponse = await emailActor.remove_access_point({
                pub_key: passkeyIdentity.getPrincipal().toText()
            });

            // Then the removal succeeds and the account email is untouched
            expect(removeResponse.status_code).eq(200);

            const account = await emailActor.get_account();
            expect(account.status_code).eq(200);
            expect(account.data[0].email).deep.eq(["ap-keep-email@test.test"]);
        });

        it("should return 200 and keep the email empty when it was already cleared", async function () {
            // Given an account whose Email access point was already removed once
            const {emailIdentity, emailActor, recoveryActor} = await setupNfidAccount(dfx, "already-empty", {
                email: "ap-already-empty@test.test",
                withPasskey: true,
                withRecovery: true
            });

            const firstRemove = await emailActor.remove_access_point({
                pub_key: emailIdentity.getPrincipal().toText()
            });
            expect(firstRemove.status_code).eq(200);

            const secondEmailIdentity = buildIdentity("ap-email-already-empty-2");
            const secondEmailPrincipal = secondEmailIdentity.getPrincipal().toText();
            const readdResponse = await recoveryActor.create_access_point({
                icon: "Icon",
                device: "Email device",
                pub_key: secondEmailPrincipal,
                browser: "Browser",
                device_type: {Email: null},
                credential_id: []
            });
            expect(readdResponse.status_code).eq(200);

            // When a freshly re-added Email access point is removed again
            const secondRemove = await recoveryActor.remove_access_point({
                pub_key: secondEmailPrincipal
            });

            // Then the removal succeeds and the account email stays empty
            expect(secondRemove.status_code).eq(200);

            const account = await recoveryActor.get_account();
            expect(account.status_code).eq(200);
            expect(account.data[0].email).deep.eq([]);
        });

        it("should return 404 without touching the email when the pub_key is unknown", async function () {
            // Given an NFID account and a principal that holds no access point on it
            const {emailActor} = await setupNfidAccount(dfx, "unknown-key", {
                email: "ap-unknown-key@test.test",
                withPasskey: true,
                withRecovery: true
            });
            const strangerPrincipal = buildIdentity("ap-stranger-unknown-key").getPrincipal().toText();

            // When removal is attempted for that unknown principal
            const removeResponse = await emailActor.remove_access_point({
                pub_key: strangerPrincipal
            });

            // Then the request is rejected as not found and the account is unchanged
            expect(removeResponse.status_code).eq(404);

            const account = await emailActor.get_account();
            expect(account.status_code).eq(200);
            expect(account.data[0].email).deep.eq(["ap-unknown-key@test.test"]);
        });
    });

});