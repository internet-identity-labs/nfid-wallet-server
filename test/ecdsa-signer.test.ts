import "mocha";
import {expect} from "chai";
import {Dfx} from "./type/dfx";
import {App} from "./constanst/app.enum";
import {deploy} from "./util/deployment.util";
import {DFX} from "./constanst/dfx.const";
import {Result, Result_1} from "./idl/ecdsa";

describe("ECDSA signer test", () => {
    describe("ECDSA tests", () => {
        var dfx: Dfx;

        before(async () => {
            dfx = await deploy({apps: [App.ECDSASigner]});
        });

        after(() => {
            DFX.STOP();
        });

        it("should return public key", async function () {
            let pk = await dfx.ecdsa.actor.public_key() as Result;
            console.log(pk)
            // @ts-ignore
            expect(pk.Ok.public_key.length > 0).eq(true)
        });

        it("should sign array", async function () {
            let message = Array(32).fill(1);
            let signature = await dfx.ecdsa.actor.sign(message) as Result_1;
            console.log(signature)
            // @ts-ignore
            expect(signature.Ok.signature.length).eq(64)
        });

    });
});
