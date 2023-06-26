import "mocha";
import {expect} from "chai";
import {sleep} from "./util/call.util";
import {Dfx} from "./type/dfx";
import {App} from "./constanst/app.enum";
import {deploy, getActor, getIdentity} from "./util/deployment.util";
import {register} from "./util/internet_identity.util";
import {
    AccessPointRequest, AccountResponse,
    BoolHttpResponse,
    HTTPAccessPointResponse,
    HTTPAccountRequest,
    HTTPAccountResponse,
} from "./idl/identity_manager";
import {DFX} from "./constanst/dfx.const";
import {idlFactory as imIdl} from "./idl/identity_manager_idl";
import {Expected} from "./constanst/expected.const";
import {Ed25519KeyIdentity} from "@dfinity/identity";
import {DeviceData} from "./idl/internet_identity_test";
import {fail} from "assert";

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

        it("should get account pn_sha2 created in previous test.", async function () {
            expect(DFX.GET_PN_SHA2("identity_manager", dfx.root)).eq(Expected.ERROR("Phone number not verified", "404"));
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
    });

    describe("Agent tests", () => {
        var dfx: Dfx;
        var iiAnchor: bigint;
        var nfidAnchor: bigint

        before(async () => {
            dfx = await deploy({apps: [App.IdentityManager, App.InternetIdentityTest]});
            iiAnchor = await register(dfx.iit.actor, dfx.user.identity);
        });

        after(() => {
            DFX.STOP();
        });

        it("should error empty device data on NFID account", async function () {
            var accountRequest: HTTPAccountRequest = {
                access_point: [],
                wallet: [{'NFID': null}],
                anchor: 0n
            };

            try {
                await dfx.im.actor.create_account(accountRequest);
                fail("Have to fail")
            } catch (e) {
                expect(e.message).contains("Device Data required");
            }
        })

        it("should create NFID account", async function () {
            const identity = getIdentity("87654321876543218765432187654311");
            const dd: AccessPointRequest = {
                icon: "Icon",
                device: "Global",
                pub_key: identity.getPrincipal().toText(),
                browser: "Browser",
                device_type: {
                    Email: null
                }
            }
            var accountRequest: HTTPAccountRequest = {
                access_point: [dd],
                wallet: [{'NFID': null}],
                anchor: 0n
            };
            const actor = await getActor(dfx.im.id, identity, imIdl);

            const accResponse: HTTPAccountResponse = (await actor.create_account(
                accountRequest
            )) as HTTPAccountResponse;
            const response = accResponse.data[0]
            nfidAnchor = response.anchor
            expect(response.anchor).eq(100000000n)
            expect(Object.keys(response.wallet)).contains("NFID")
            expect(response.access_points.length).eq(1)
            expect(response.personas.length).eq(0)
        })

        it("should throw error due to not authentificated principal on creating account.", async function () {
            var accountRequest: HTTPAccountRequest = {
                access_point: [], wallet: [],
                anchor: iiAnchor + 1n
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
                anchor: iiAnchor
            };

            var response: HTTPAccountResponse = (await dfx.im.actor.create_account(
                accountRequest
            )) as HTTPAccountResponse;
            expect(response.status_code).eq(200);
            expect(response.data[0].anchor).eq(iiAnchor);
            expect(response.error).empty;
        });

        it("should throw error due to not authentificated principal on creating access point.", async function () {
            var request: AccessPointRequest = {
                icon: "icon",
                device: "device",
                pub_key: Ed25519KeyIdentity.generate().getPrincipal().toText(),
                browser: "browser",
                device_type: {
                    Email: null
                }
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
                    Email: null
                }
            };

            var response: HTTPAccessPointResponse = (await dfx.im.actor.create_access_point(
                request
            )) as HTTPAccessPointResponse;
            expect(response.status_code).eq(200);

            var point = response.data[0][0];
            expect(point.icon).eq("icon");
            expect(point.device).eq("device");
            expect(point.browser).eq("browser");

            expect(response.error).empty;
        });

        it("should recover account.", async function () {
            var response: HTTPAccountResponse = (await dfx.im.actor.recover_account(
                iiAnchor, []
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
            var response: HTTPAccountResponse = await actor.recover_account(nfidAnchor, [{'NFID': null}]) as HTTPAccountResponse;

            expect(response.status_code).eq(200);
            expect(response.data[0].anchor).eq(100000000n);
            expect(response.error).empty;
        });

        it("should backup and restore account.", async function () {
            let anchorNew = await register(dfx.iit.actor, dfx.user.identity);
            var accountRequest: HTTPAccountRequest = {
                anchor: anchorNew,
                access_point: [],
                wallet: []
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
            const identity = Ed25519KeyIdentity.generate();
            const dd: AccessPointRequest = {
                icon: "Icon",
                device: "Global",
                pub_key: identity.getPrincipal().toText(),
                browser: "Browser",
                device_type: {
                    Email: null
                }
            }
            var accountRequest: HTTPAccountRequest = {
                access_point: [dd],
                wallet: [{'NFID': null}],
                anchor: 0n
            };
            const actor = await getActor(dfx.im.id, identity, imIdl);
            await actor.create_account(
                accountRequest
            )
            const identityDevice = getIdentity("87654321876543218765432187654123");
            const deviceData2: AccessPointRequest = {
                icon: "Icon",
                device: "Global",
                pub_key: identityDevice.getPrincipal().toText(),
                browser: "Browser",
                device_type: {
                    Passkey: null
                }
            }
            await actor.create_access_point(deviceData2)

            //enable 2fa
            let account = (await actor.update_2fa(true)) as AccountResponse
            expect(account.is2fa_enabled).eq(true)
            const actorDevice = await getActor(dfx.im.id, identityDevice, imIdl);
            //try to update from Email
            try {
                await actor.update_2fa(false)
            }catch (e) {
                expect(e.message).contains("Unauthorised");
            }
            let root = (await actorDevice.get_root_by_principal(identityDevice.getPrincipal().toText())) as string
            expect(root[0]).eq(identity.getPrincipal().toText())
            try{
                await actorDevice.get_root_by_principal(identity.getPrincipal().toText())
                fail("Should Fail")
            }catch (e) {
                expect(e.message).contains("Unauthorised");
            }
            let updated2fa = (await actorDevice.update_2fa(false)) as AccountResponse
            expect(updated2fa.is2fa_enabled).eq(false)
        });
    });
});
