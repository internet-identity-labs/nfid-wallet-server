import "mocha";
import {call} from "./util/call.util";
import {idlFactory as idl} from "./idl/vault_idl";
import {getActor, getIdentity} from "./util/deployment.util";
import {DFX} from "./constanst/dfx.const";

describe("Asd", () => {
    describe("Asd tests", async function () {
        let a = call(`./demo.sh`)
        console.log(a)
        let identity = getIdentity("87654321876543218765432187654321");
        let vault = DFX.GET_CANISTER_ID("vault");
        console.log(vault)
        let actor = await getActor(vault, identity, idl);
        console.log(555)
        let aaa = await actor.sub(vault, 1, 1);
        console.log(aaa)
    });
});
