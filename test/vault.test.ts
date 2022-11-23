import "mocha";
import {call} from "./util/call.util";
import {idlFactory as idl} from "./idl/vault_idl";
import {getActor, getIdentity} from "./util/deployment.util";
import {DFX} from "./constanst/dfx.const";

describe("Asd", () => {
    describe("Asd tests", async function () {
        console.log(321)

        console.log(call(`./demo.sh`))
        console.log(1234)
        console.log(call(`./demo2.sh`))

        let identity = getIdentity("87654321876543218765432187654321");
        let vault = DFX.GET_CANISTER_ID("vault");
        console.log(vault)
        let actor = await getActor(vault, identity, idl);
        console.log(555)
        let aaa = await actor.sub(vault, 1, 1);
        console.log(aaa)
        let group = await actor.register_group("groupName1")
        console.log("group")
        console.log("group")
        console.log("group")
        console.log("group")
        console.log(group)

    });
});
