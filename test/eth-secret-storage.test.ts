import "mocha";
import {expect} from "chai";
import {Dfx} from "./type/dfx";
import {App} from "./constanst/app.enum";
import {deploy} from "./util/deployment.util";
import {DFX} from "./constanst/dfx.const";

const ADDRESS = "0xdc75e8c3ae765d8947adbc6698a2403a6141d439";
const SIGNATURE = "0x2abb0d6e433694a9b03bbc7d3d2e9ab713cdfab9f47cb92378ea930a4357e4712277c3a987d950e052f27f12331d8882e8a1c7fdc8886aae34b58189df9488751b";
const ADDRESS2 = "0x05b0901b659a2dcc41dc6bd7a333a25597a3527e";
const SIGNATURE2 = "0xccfd9eb0bf034622dd40bfde6e2debfab16f36a2b715444f86a26507131cd06d2d92e12bc6d49a181a8f2a179385e555eb18f05731b71f153e3de4c46936edec1b";
const SIGNATURE3 = "0x90069f397055f97fda932e22a15eaa80a8c4f827a0a777c1005a6e1d8dd5553f116421c402e4334d9aa649b0879c697ec0fa2b2143012632cb0572c7de86d07a1b";

describe.skip("Eth Secret Storage", () => {
    describe("Agent tests", () => {
        var dfx: Dfx;

        before(async () => {
            dfx = await deploy({apps: [App.EthSecretStorage]});
        });

        after(() => {
            DFX.STOP();
        });

        it("should return signature verification failed error", async function () {
            try {
                await dfx.ess.actor.get_secret(ADDRESS, SIGNATURE3);
            } catch(error) {
                expect(JSON.stringify(error)).contains(`Signature verification failed. Expected 0xdc75…d439, got 0xf94e…3ff8`);
            }
        });

        it("should return incorrect signature odd number of digits error", async function () {
            try {
                await dfx.ess.actor.get_secret(ADDRESS, "0xa") as String;
            } catch(error) {
                expect(JSON.stringify(error)).contains(`Incorrect signature: Odd number of digits`);
            }
        });

        it("should return invalid signature length error", async function () {
            try {
                await dfx.ess.actor.get_secret(ADDRESS, "0xaa") as String;
            } catch(error) {
                expect(JSON.stringify(error)).contains(`Incorrect signature: invalid signature length, got 1, expected 65`);
            }
        });
        
        it("should return incorrect address error", async function () {
            try {
                await dfx.ess.actor.get_secret("0xa", SIGNATURE) as String;
            } catch(error) {
                expect(JSON.stringify(error)).contains(`Incorrect address: Odd number of digits`);
            }
        });

        it("should return incorrect address lengh error", async function () {
            try {
                await dfx.ess.actor.get_secret("0xaa", SIGNATURE) as String;
            } catch(error) {
                expect(JSON.stringify(error)).contains(`Incorrect address lengh`);
            }
        });

        it("should return same secret for the same signature", async function () {
            const [secret1, secret2, secret3] = await Promise.all([
                dfx.ess.actor.get_secret(ADDRESS, SIGNATURE), 
                dfx.ess.actor.get_secret(ADDRESS, SIGNATURE),
                dfx.ess.actor.get_secret(ADDRESS2, SIGNATURE2)
            ]) as [String, String, String];

            expect(secret1.length).eq(66);
            expect(secret2.length).eq(66);
            expect(secret3.length).eq(66);

            expect(secret1).eq(secret2);
            expect(secret1).not.eq(secret3);
        });

        it("should return same secret for the same address and signature after upgrade", async function () {
            const secret1 = await dfx.ess.actor.get_secret(ADDRESS, SIGNATURE) as String;

            dfx = await deploy({clean: false, apps: [App.EthSecretStorage]});

            const secret2 = await dfx.ess.actor.get_secret(ADDRESS, SIGNATURE) as String;

            expect(secret1.length).eq(66);
            expect(secret2.length).eq(66);

            expect(secret1).eq(secret2);
        });
    });
});
