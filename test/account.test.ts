import "mocha";
import {expect} from "chai";
import {sleep} from "./util/call.util";
import {Dfx} from "./type/dfx";
import {App} from "./constanst/app.enum";
import {deploy, getActor, getIdentity} from "./util/deployment.util";
import {register} from "./util/internet_identity.util";
import {
    AccessPointRequest,
    BoolHttpResponse,
    HTTPAccessPointResponse,
    HTTPAccountRequest,
    HTTPAccountResponse,
} from "./idl/identity_manager";
import {DeviceData} from "./idl/internet_identity_test";
import {DFX} from "./constanst/dfx.const";
import {idlFactory as imIdl} from "./idl/identity_manager_idl";
import {Expected} from "./constanst/expected.const";

const PHONE = "123456";
const PHONE_SHA2 = "123456_SHA2";
const TOKEN = "1234";

describe("Account", () => {
    describe("CLI tests", () => {
        var dfx: Dfx;

        before(async () => {
            dfx = await deploy(App.IdentityManager, App.IdentityManagerReplica);
        });

        after(() => {
            DFX.STOP();
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
        var anchor: bigint;

        before(async () => {
            dfx = await deploy(App.IdentityManager, App.InternetIdentityTest);
            anchor = await register(dfx.iit.actor, dfx.user.identity);
        });

        after(() => {
            DFX.STOP();
        });

        it("should throw error due to not authentificated principal on creating account.", async function () {
            var accountRequest: HTTPAccountRequest = {
                anchor: anchor + 1n,
            };

            try {
                await dfx.im.actor.create_account(accountRequest);
            } catch (e) {
                expect(e.message).contains("could not be authenticated");
            }
        });

        it("should create account.", async function () {
            var accountRequest: HTTPAccountRequest = {
                anchor: anchor,
            };

            var response: HTTPAccountResponse = (await dfx.im.actor.create_account(
                accountRequest
            )) as HTTPAccountResponse;
            expect(response.status_code).eq(200);
            expect(response.data[0].anchor).eq(anchor);
            expect(response.error).empty;
        });

        it("should throw error due to not authentificated principal on creating access point.", async function () {
            var pbk = Array.from(new Uint8Array(dfx.user.identity.getPublicKey().toDer()));
            pbk[0] = 0;
            var request: AccessPointRequest = {
                icon: "icon",
                device: "device",
                pub_key: pbk,
                browser: "browser",
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
                pub_key: Array.from(new Uint8Array(dfx.user.identity.getPublicKey().toDer())),
                browser: "browser",
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
                anchor
            )) as HTTPAccountResponse;
            expect(response.status_code).eq(200);
            expect(response.data[0].anchor).eq(anchor);
            expect(response.error).empty;
        });

        it("should throw error due to not existing anchor.", async function () {
            try {
                await dfx.im.actor.recover_account(anchor + 1n);
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

            await dfx.iit.actor.add(anchor, deviceData);

            await actor.recover_account(anchor);
            var response: HTTPAccountResponse = (await actor.get_account()) as HTTPAccountResponse;

            expect(response.status_code).eq(200);
            expect(response.data[0].anchor).eq(anchor);
            expect(response.error).empty;
        });

        it("should backup and restore account.", async function () {
            let  anchorNew = await register(dfx.iit.actor, dfx.user.identity);
            var accountRequest: HTTPAccountRequest = {
                anchor: anchorNew,
            };
            await dfx.im.actor.create_account(accountRequest);
            const backup = await dfx.im.actor.get_all_accounts_json();
            const remove = (await dfx.im.actor.remove_account()) as BoolHttpResponse;
            expect(remove.data[0]).eq(true);
            var response: HTTPAccountResponse = (await dfx.im.actor.get_account()) as HTTPAccountResponse;
            expect(response.error[0]).eq("Unable to find Account");
            await dfx.im.actor.add_all_accounts_json(backup);
            response = (await dfx.im.actor.get_account()) as HTTPAccountResponse;
            expect(response.error).empty;
            expect(response.data[0].anchor).eq(anchorNew);
        });
    });
});
