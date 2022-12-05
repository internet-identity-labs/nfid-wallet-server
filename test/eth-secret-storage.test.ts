import "mocha";
import {expect} from "chai";
import {Dfx} from "./type/dfx";
import {App} from "./constanst/app.enum";
import {deploy} from "./util/deployment.util";
import {DFX} from "./constanst/dfx.const";
import {StringHttpResponse} from "./idl/eth_secret_storage";

const SIGNATURE = "0x53ee21fc662cef31d21b64e42e76694d170d2a419de74459d95d33210911b45e2935e0b22ec8d09c4ff8fc22cc534568bc2cc0772303c6ad8a959266a568d6c91b";
const SIGNATURE2 = "0x90069f397055f97fda932e22a15eaa80a8c4f827a0a777c1005a6e1d8dd5553f116421c402e4334d9aa649b0879c697ec0fa2b2143012632cb0572c7de86d07a1b";
const APP = "METAMASK";

describe("Eth Secret Storge", () => {
    describe("Agent tests", () => {
        var dfx: Dfx;

        before(async () => {
            dfx = await deploy({apps: [App.EthSecretStorage]});
        });

        after(() => {
            DFX.STOP();
        });

        it("should return no such app error", async function () {
            const secret = await dfx.ess.actor.secret_by_signature("INCORRECT_NAME", SIGNATURE) as StringHttpResponse;

            expect(secret).deep.eq({
                data: [],
                error: ["No such application."],
                status_code: 400
            });
        });

        it("should return odd number of digits error", async function () {
            const secret = await dfx.ess.actor.secret_by_signature(APP, "a") as StringHttpResponse;

            expect(secret).deep.eq({
                data: [],
                error: ["Odd number of digits"],
                status_code: 400
            });
        });

        it("should return incorrect signature lenght error", async function () {
            const secret = await dfx.ess.actor.secret_by_signature(APP, "aa") as StringHttpResponse;

            expect(secret).deep.eq({
                data: [],
                error: ["invalid signature length, got 1, expected 65"],
                status_code: 400
            });
        });

        it("should return same secret for the same signature", async function () {
            const secret1 = await dfx.ess.actor.secret_by_signature(APP, SIGNATURE) as StringHttpResponse;
            const secret2 = await dfx.ess.actor.secret_by_signature(APP, SIGNATURE) as StringHttpResponse;
            const secret3 = await dfx.ess.actor.secret_by_signature(APP, SIGNATURE2) as StringHttpResponse;

            expect(secret1.status_code).eq(200);
            expect(secret2.status_code).eq(200);
            expect(secret3.status_code).eq(200);

            expect(secret1.error).empty;
            expect(secret2.error).empty;
            expect(secret3.error).empty;

            expect(secret1.data[0]).eq(secret2.data[0]);
            expect(secret1.data[0]).not.eq(secret3.data[0]);
        });

        it("should return same secret for the same signature after upgrade", async function () {
            const secret1 = await dfx.ess.actor.secret_by_signature(APP, SIGNATURE) as StringHttpResponse;

            dfx = await deploy({clean: false, apps: [App.EthSecretStorage]});

            const secret2 = await dfx.ess.actor.secret_by_signature(APP, SIGNATURE) as StringHttpResponse;

            expect(secret1.status_code).eq(200);
            expect(secret2.status_code).eq(200);

            expect(secret1.error).empty;
            expect(secret2.error).empty;

            expect(secret1.data[0]).eq(secret2.data[0]);
        });
    });
});
