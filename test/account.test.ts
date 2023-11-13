import "mocha";
import { expect } from "chai";
import { sleep } from "./util/call.util";
import { Dfx } from "./type/dfx";
import { App } from "./constanst/app.enum";
import { deploy, getActor, getIdentity } from "./util/deployment.util";
import { register } from "./util/internet_identity.util";
import {
    AccessPointRequest,
    AccountResponse,
    BoolHttpResponse, CertifiedResponse,
    HTTPAccessPointResponse,
    HTTPAccountRequest,
    HTTPAccountResponse,
    HTTPAccountUpdateRequest,
} from "./idl/identity_manager";
import { DFX } from "./constanst/dfx.const";
import { idlFactory as imIdl } from "./idl/identity_manager_idl";
import { Expected } from "./constanst/expected.const";
import { Ed25519KeyIdentity } from "@dfinity/identity";
import { DeviceData } from "./idl/internet_identity_test";
import { fail } from "assert";

const PHONE = "123456";
const PHONE_SHA2 = "123456_SHA2";
const TOKEN = "1234";

describe("Account", () => {
    describe("CLI tests", () => {
        var dfx: Dfx;

        before(async () => {
            dfx = await deploy({apps: [App.IdentityManager, App.IdentityManagerReplica]});
        });

        after(() => {
            DFX.STOP();
            DFX.KILL_PORT();
        });

        it("should create correct account.", async function () {
            expect(DFX.CREATE_ACCOUNT("12345")).eq(Expected.ACCOUNT("null", dfx.root));
        });

        it("should get created account from previous test.", async function () {
            expect(DFX.GET_ACCOUNT("identity_manager")).eq(Expected.ACCOUNT("null", dfx.root));
        });

        it("should get created account by principal from previous test.", async function () {
            expect(DFX.GET_ACCOUNT_BY_PRINCIPAL("identity_manager", dfx.root)).eq(Expected.ACCOUNT("null", dfx.root));
        });

        it("should update account name.", async function () {
            expect(DFX.UPDATE_ACCOUNT_NAME()).eq(Expected.ACCOUNT(`opt "TEST_USER_UPDATED"`, dfx.root));
        });

        it("should throw error due to existing anchor.", async function () {
            DFX.TOKEN(PHONE, PHONE_SHA2, TOKEN, dfx.root);
            expect(DFX.CREATE_ACCOUNT_FULL()).eq(Expected.ERROR("Impossible to link this II anchor, please try another one.", "404"));
        });

        it("should return replicated copy of account on heartbeat rate.", async function () {
            await sleep(3);
            expect(DFX.GET_ACCOUNT("identity_manager_replica")).eq(Expected.ACCOUNT(`opt "TEST_USER_UPDATED"`, dfx.root));
        });

        it("should recover account.", async function () {
            expect(DFX.RECOVER_ACCOUNT()).eq(Expected.ACCOUNT(`opt "TEST_USER_UPDATED"`, dfx.root));
        });

        it("should restore account by api call.", async function () {
            expect(DFX.REMOVE_ACCOUNT("identity_manager_replica")).eq(Expected.BOOL("true", "200"));
            expect(DFX.GET_ACCOUNT("identity_manager_replica")).eq(Expected.ERROR("Unable to find Account", "404"));
            DFX.RESTORE_ACCOUNT("identity_manager", dfx.imr.id);
            expect(DFX.GET_ACCOUNT("identity_manager_replica")).eq(Expected.ACCOUNT(`opt "TEST_USER_UPDATED"`, dfx.root));
        });

        it("should remove account and create new one.", async function () {
            expect(DFX.REMOVE_ACCOUNT("identity_manager")).eq(Expected.BOOL("true", "200"));
            expect(DFX.REMOVE_ACCOUNT("identity_manager")).eq(Expected.ERROR("Unable to remove Account", "404"));
            expect(DFX.CREATE_ACCOUNT_2()).eq(Expected.ACCOUNT("null", dfx.root));
        });

        it("should update email and receive an error.", async function () {
            expect(DFX.UPDATE_ACCOUNT_EMAIL("test@test.test")).contains("Email and principal are not valid.");
        });

        it("should remove account and create new one with email and receive an error.", async function () {
            expect(DFX.REMOVE_ACCOUNT("identity_manager")).eq(Expected.BOOL("true", "200"));
            expect(DFX.CREATE_ACCOUNT_WITH_EMAIL("12345", 'opt "test@test.test"')).contains("Email and principal are not valid.");
        });
    });

    describe("Agent tests", () => {
        var dfx: Dfx;
        var iiAnchor: bigint;
        var nfidAnchor: bigint;

        before(async () => {
            dfx = await deploy({ apps: [App.IdentityManager, App.InternetIdentityTest] });
            iiAnchor = await register(dfx.iit.actor, dfx.user.identity);
        });

        after(() => {
            DFX.STOP();
        });

        it("should return an error empty device data on NFID account", async function () {
            let response = await dfx.im.actor.add_email_and_principal_for_create_account_validation("testdefault@test.test", dfx.user.principal, 25) as BoolHttpResponse;
            expect(response.status_code).eq(200);

            var accountRequest: HTTPAccountRequest = {
                access_point: [],
                wallet: [{ NFID: null }],
                anchor: 0n,
                email: ["testdefault@test.test"],
            };

            try {
                await dfx.im.actor.create_account(accountRequest);
                fail("Have to fail");
            } catch (e) {
                expect(e.message).contains("Device Data required");
            }
        });

        it("should return an error by adding new email with bigger timestamp due to self cleaning of the service", async function () {
            let response1 = await dfx.im.actor.add_email_and_principal_for_create_account_validation("testdefault@test.test", dfx.user.principal, 900000) as BoolHttpResponse;
            expect(response1.status_code).eq(200);
            
            let response2 = await dfx.im.actor.add_email_and_principal_for_create_account_validation("1@test.test", dfx.user.principal, 900000 * 2 + 1) as BoolHttpResponse;
            expect(response2.status_code).eq(200);

            var accountRequest: HTTPAccountRequest = {
                access_point: [],
                wallet: [{ NFID: null }],
                anchor: 0n,
                email: ["testdefault@test.test"],
            };

            try {
                await dfx.im.actor.create_account(accountRequest);
                fail("Have to fail");
            } catch (e) {
                console.log({e})
                expect(e.message).contains("Email and principal are not valid");
            }
        });

        it("should create NFID account", async function () {
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
                wallet: [{ NFID: null }],
                anchor: 0n,
                email: ["test@test.test"],
            };
            const actor = await getActor(dfx.im.id, identity, imIdl);

            let email_response = await dfx.im.actor.add_email_and_principal_for_create_account_validation("test@test.test", principal, 25) as BoolHttpResponse;
            expect(email_response.status_code).eq(200);

            const accResponse: HTTPAccountResponse = (await actor.create_account(
                accountRequest
            )) as HTTPAccountResponse;
            const response = accResponse.data[0];
            nfidAnchor = response.anchor;
            expect(response.anchor).eq(100000000n);
            expect(Object.keys(response.wallet)).contains("NFID");
            expect(response.access_points.length).eq(1);
            expect(response.personas.length).eq(0);
            expect(response.email[0]).contains("test@test.test");
            var certifiedResponse = (await actor.get_root_certified())as CertifiedResponse
            expect(certifiedResponse.witness.length > 0).eq(true)
            expect(certifiedResponse.response).eq(identity.getPrincipal().toText())
        });

        it("should throw error due to not authentificated principal on creating account.", async function () {
            var accountRequest: HTTPAccountRequest = {
                access_point: [],
                wallet: [],
                anchor: iiAnchor + 1n,
                email: [],
            };
            try {
                await dfx.im.actor.create_account(accountRequest);
            } catch (e) {
                expect(e.message).contains("could not be authenticated");
            }
        });

        it("should create account.", async function () {
            var accountRequest: HTTPAccountRequest = {
                access_point: [],
                wallet: [],
                anchor: iiAnchor,
                email: [],
            };

            var response: HTTPAccountResponse = (await dfx.im.actor.create_account(
                accountRequest
            )) as HTTPAccountResponse;
            expect(response.status_code).eq(200);
            expect(response.data[0].anchor).eq(iiAnchor);
            expect(response.error).empty;
        });

        it("should create II account with no email and update email of it.", async function () {
            var updateRequest: HTTPAccountUpdateRequest = {
                name: [],
                email: ["testdefault@test.test"]
            }

            const updateResponse: HTTPAccountResponse = (await dfx.im.actor.update_account(
                updateRequest
            )) as HTTPAccountResponse;

            expect(updateResponse.data[0].email[0]).contains("testdefault@test.test");
        });

        it("should try to create account and receive incorrect email and principal when incorrect email.", async function () {
            var accountRequest: HTTPAccountRequest = {
                access_point: [],
                wallet: [{ NFID: null }],
                anchor: iiAnchor,
                email: ["invalid@test.test"],
            };

            try {
                await dfx.im.actor.create_account(accountRequest);
            } catch (e) {
                expect(e.message).contains("Email and principal are not valid");
            }
        });

        it("should try to create account and receive incorrect email and principal when incorrect principal.", async function () {
            var accountRequest: HTTPAccountRequest = {
                access_point: [],
                wallet: [{ NFID: null }],
                anchor: iiAnchor,
                email: ["test@test.test"],
            };

            try {
                await dfx.im.actor.create_account(accountRequest);
            } catch (e) {
                expect(e.message).contains("Email and principal are not valid.");
            }
        });

        it("should throw error due to not authentificated principal on creating access point.", async function () {
            var request: AccessPointRequest = {
                icon: "icon",
                device: "device",
                pub_key: Ed25519KeyIdentity.generate().getPrincipal().toText(),
                browser: "browser",
                device_type: {
                    Email: null,
                },
                credential_id: [],
            };

            try {
                await dfx.im.actor.create_access_point(request);
            } catch (e) {
                expect(e.message).contains("could not be authenticated");
            }
        });

        it("should create access point.", async function () {
            var request: AccessPointRequest = {
                icon: "icon",
                device: "device",
                pub_key: dfx.user.identity.getPrincipal().toText(),
                browser: "browser",
                device_type: {
                    Email: null,
                },
                credential_id: ["test_id"],
            };

            var response: HTTPAccessPointResponse = (await dfx.im.actor.create_access_point(
                request
            )) as HTTPAccessPointResponse;
            expect(response.status_code).eq(200);

            var point = response.data[0][0];
            expect(point.icon).eq("icon");
            expect(point.device).eq("device");
            expect(point.browser).eq("browser");
            expect(point.credential_id[0]).eq("test_id");

            expect(response.error).empty;
        });

        it("should recover account.", async function () {
            var response: HTTPAccountResponse = (await dfx.im.actor.recover_account(
                iiAnchor,
                []
            )) as HTTPAccountResponse;
            expect(response.status_code).eq(200);
            expect(response.data[0].anchor).eq(iiAnchor);
            expect(response.error).empty;
        });

        it("should throw error due to not existing anchor.", async function () {
            try {
                await dfx.im.actor.recover_account(iiAnchor + 1n, []);
            } catch (e) {
                expect(e.message).contains("could not be authenticated");
            }
        });

        it("should remove account.", async function () {
            var response: BoolHttpResponse = (await dfx.im.actor.remove_account()) as BoolHttpResponse;
            expect(response.status_code).eq(200);
            expect(response.data[0]).eq(true);
            expect(response.error).empty;
        });

        it("should recover account using seed phrase.", async function () {
            const identity = getIdentity("87654321876543218765432187654322");
            const actor = await getActor(dfx.im.id, identity, imIdl);
            var deviceData: DeviceData = {
                alias: "RecoveryDevice",
                protection: {
                    protected: null,
                },
                pubkey: Array.from(new Uint8Array(identity.getPublicKey().toDer())),
                key_type: {
                    seed_phrase: null,
                },
                purpose: {
                    authentication: null,
                },
                credential_id: [],
            };

            await dfx.iit.actor.add(iiAnchor, deviceData);

            await actor.recover_account(iiAnchor, []);
            var response: HTTPAccountResponse = (await actor.get_account()) as HTTPAccountResponse;

            expect(response.status_code).eq(200);
            expect(response.data[0].anchor).eq(iiAnchor);
            expect(response.error).empty;
        });

        it("should recover NFID account using registered device.", async function () {
            const identity = getIdentity("87654321876543218765432187654311");
            const actor = await getActor(dfx.im.id, identity, imIdl);
            var response: HTTPAccountResponse = (await actor.recover_account(nfidAnchor, [
                { NFID: null },
            ])) as HTTPAccountResponse;

            expect(response.status_code).eq(200);
            expect(response.data[0].anchor).eq(100000000n);
            expect(response.error).empty;
        });

        it("should backup and restore account.", async function () {
            let anchorNew = await register(dfx.iit.actor, dfx.user.identity);
            var accountRequest: HTTPAccountRequest = {
                anchor: anchorNew,
                access_point: [],
                wallet: [],
                email: [],
            };
            await dfx.im.actor.create_account(accountRequest as any);
            const backup = await dfx.im.actor.get_all_accounts_json(0, 5);
            const remove = (await dfx.im.actor.remove_account()) as BoolHttpResponse;
            expect(remove.data[0]).eq(true);
            var response: HTTPAccountResponse = (await dfx.im.actor.get_account()) as HTTPAccountResponse;
            expect(response.error[0]).eq("Unable to find Account");
            await dfx.im.actor.add_all_accounts_json(backup);
            response = (await dfx.im.actor.get_account()) as HTTPAccountResponse;
            expect(response.error).empty;
            expect(response.data[0].anchor).eq(anchorNew);
            expect(Object.keys(response.data[0].wallet)).contains("II");
        });

        it("should enable 2fa", async function () {
            const identity = getIdentity("87654321876543218765432187654411");
            const dd: AccessPointRequest = {
                icon: "Icon",
                device: "Global",
                pub_key: identity.getPrincipal().toText(),
                browser: "Browser",
                device_type: {
                    Email: null,
                },
                credential_id: [],
            };
            var accountRequest: HTTPAccountRequest = {
                access_point: [dd],
                wallet: [{ NFID: null }],
                anchor: 0n,
                email: ["test2@test.test"],
            };
            const actor = await getActor(dfx.im.id, identity, imIdl);

            const principal = identity.getPrincipal().toString();
            let email_response = await dfx.im.actor.add_email_and_principal_for_create_account_validation("test2@test.test", principal, 25) as BoolHttpResponse;
            expect(email_response.status_code).eq(200);

            await actor.create_account(accountRequest);
            const identityDevice = getIdentity("87654321876543218765432187654123");
            const deviceData2: AccessPointRequest = {
                icon: "Icon",
                device: "Global",
                pub_key: identityDevice.getPrincipal().toText(),
                browser: "Browser",
                device_type: {
                    Passkey: null,
                },
                credential_id: ["some_id"],
            };
            await actor.create_access_point(deviceData2);

            //enable 2fa
            let account = (await actor.update_2fa(true)) as AccountResponse;
            expect(account.is2fa_enabled).eq(true);
            const actorDevice = await getActor(dfx.im.id, identityDevice, imIdl);
            //try to update from Email
            try {
                await actor.update_2fa(false);
            } catch (e) {
                expect(e.message).contains("Unauthorised");
            }
            let root = (await actorDevice.get_root_by_principal(identityDevice.getPrincipal().toText())) as string;
            expect(root[0]).eq(identity.getPrincipal().toText());
            try {
                await actorDevice.get_root_by_principal(identity.getPrincipal().toText());
                fail("Should Fail");
            } catch (e) {
                expect(e.message).contains("Unauthorised");
            }
            let updated2fa = (await actorDevice.update_2fa(false)) as AccountResponse;
            expect(updated2fa.is2fa_enabled).eq(false);
        });
    });
});
