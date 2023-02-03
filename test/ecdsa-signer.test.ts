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

        it("should prepare signature", async function () {
            let message = Array(32).fill(1);
            let key = await dfx.ecdsa.actor.prepare_signature(message) as String;
            expect(key).eq("")
            let signature = await dfx.ecdsa.actor.get_signature(key) as Result_1;
            console.log(signature)
            // @ts-ignore
            expect(signature.Ok.signature.length).eq(64)
            let signature2 = await dfx.ecdsa.actor.get_signature(key) as Result_1;
            console.log(signature2)
            // @ts-ignore
            expect(assertArray(signature.Ok.signature, signature2.Ok.signature)).eq(true)
            await new Promise(r => setTimeout(r, 3000));
            await dfx.ecdsa.actor.public_key() as Result; //run cleanup
            let signature3 = await dfx.ecdsa.actor.get_signature(key) as Result_1;
            // @ts-ignore
            expect(signature3.Err).eq('No such signature')
        });
    });

    function assertArray(a: [], b: []) {
        let i = 0;
        while (i < 64) {
            if (a[i] !== b[i]) {
                return false
            }
            i++
        }
        return true
    }
});
