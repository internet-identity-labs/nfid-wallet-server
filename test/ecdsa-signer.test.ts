import "mocha";
import {expect} from "chai";
import {Dfx} from "./type/dfx";
import {App} from "./constanst/app.enum";
import {deploy} from "./util/deployment.util";
import {DFX} from "./constanst/dfx.const";
import {KeyPair, KeyPairResponse} from "./idl/ecdsa";
import {fail} from "assert";

describe("ECDSA signer test", () => {
    describe("ECDSA tests", () => {
        var dfx: Dfx;

        before(async () => {
            dfx = await deploy({apps: [App.ECDSASigner]});
        });

        after(() => {
            DFX.STOP();
        });

        it("should return key pair", async function () {
            let kp: KeyPair = {
                private_key_encrypted: "test_private", public_key: "test_public"
            }
            let emptyResponse = await dfx.eth_signer.actor.get_kp() as KeyPairResponse;
            expect(emptyResponse.key_pair.length).eq(0)
            await dfx.eth_signer.actor.add_kp(kp);
            try {
                await dfx.eth_signer.actor.add_kp(kp);
            } catch (e) {
                expect(e.message.includes("Already registered"))
            }
            let response = await dfx.eth_signer.actor.get_kp() as KeyPairResponse;
            expect(response.key_pair[0].public_key).eq("test_public")
            expect(response.key_pair[0].private_key_encrypted).eq("test_private")
            expect(response.princ).eq(dfx.user.identity.getPrincipal().toText())
            DFX.UPGRADE_FORCE('ecdsa_signer')
            response = await dfx.eth_signer.actor.get_kp() as KeyPairResponse;
            expect(response.key_pair[0].public_key).eq("test_public")
            expect(response.key_pair[0].private_key_encrypted).eq("test_private")
        });

        it("should backup", async function () {
            try {
                await dfx.eth_signer.actor.get_all_json(0, 10)
                fail("Should unauthorised")
            } catch (e) {
                expect(e.message).contains("Unauthorised")
                DFX.USE_TEST_ADMIN();
                DFX.ADD_CONTROLLER(dfx.user.identity.getPrincipal().toText(), "ecdsa_signer");
                DFX.ADD_CONTROLLER(dfx.eth_signer.id, "ecdsa_signer");
            }
            await dfx.eth_signer.actor.sync_controllers()
            let count = await dfx.eth_signer.actor.count()
            expect(count).eq(1n)
            let json = await dfx.eth_signer.actor.get_all_json(0, 10)
            expect(json).contains("public_key")
            expect(json).contains("test_public")
            expect(json).contains("private_key")
            expect(json).contains("test_private")
            expect(json).contains("principal")
            expect(json).contains("created_date")
        });
    });
});
