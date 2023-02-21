import "mocha";
import {expect} from "chai";
import {Dfx} from "./type/dfx";
import {App} from "./constanst/app.enum";
import {deploy} from "./util/deployment.util";
import {DFX} from "./constanst/dfx.const";
import {Result, Result_1} from "./idl/ecdsa";
import {bitcoin_address, satoshi} from "./idl/bitcoin";


describe.skip("BTC", () => {
    describe("BTC tests", () => {
        var dfx: Dfx;

        before(async () => {
            dfx = await deploy({apps: [App.BTC]});
        });

        after(() => {
            DFX.STOP();
        });

        it("should return address", async function () {
            let pk = await dfx.btc.actor.get_p2pkh_address() as String;
            expect(pk.length).eq(34)
        });

        it("should sign array", async function () {
            try {
                await dfx.btc.actor.send({
                    'destination_address': "mo5LDPFpqDzpk3FWrWDTU4t42ncjWaNPdT",
                    'amount_in_satoshi': 100000,
                })
            } catch (e) {
                expect(e.message).contains("Insufficient balance")
            }
        });
    });
});
