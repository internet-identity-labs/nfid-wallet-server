import { Actor, ActorMethod, HttpAgent, Identity } from "@dfinity/agent";
import { Ed25519KeyIdentity } from "@dfinity/identity";
import { Dfx } from "../type/dfx";
import { idlFactory as imIdl } from "../idl/identity_manager_idl";
import { idlFactory as iitIdl } from "../idl/internet_identity_test_idl";
import { TextEncoder } from "util";
import { App } from "../constanst/app.enum";
import { IDL } from "@dfinity/candid";
import { DFX } from "../constanst/dfx.const";

const localhost: string = "http://127.0.0.1:8000";

export const deploy = async (...apps: App[]): Promise<Dfx> => {
    var i = 0;
    var imConfigurationArguments = [];
    var dfx: Dfx = {
        root: null,
        user: {
            principal: null,
            identity: null
        },
        im: {
            id: null,
            actor: null,
        },
        imr: {
            id: null,
        },
        iit: {
            id: null,
            actor: null,
            anchor: null,
        },
    };

    while (++i <= 5) {
        dfx.user.identity = getIdentity("87654321876543218765432187654321");
        dfx.user.principal = dfx.user.identity.getPrincipal().toString();

        DFX.STOP();
        DFX.REMOVE_DFX_FOLDER();
        DFX.CREATE_TEST_PERSON();
        DFX.USE_TEST_ADMIN();

        dfx.root = DFX.GET_PRINCIPAL();
        DFX.INIT();

        if (apps.includes(App.IdentityManager)) {
            DFX.DEPLOY("identity_manager");

            var response = DFX.CONFIGURE();

            if (response !== "()") {
                continue;
            }

            dfx.im.id = DFX.GET_CANISTER_ID("identity_manager");
            console.debug(">> ", dfx.im.id);

            dfx.im.actor = await getActor(dfx.im.id, dfx.user.identity, imIdl);

            DFX.ADD_CONTROLLER(dfx.im.id, "identity_manager");
            DFX.ADD_CONTROLLER(dfx.user.principal, "identity_manager");
            DFX.SYNC_CONTROLLER();

            if (!apps.includes(App.InternetIdentityTest)) {
                imConfigurationArguments.push(`env = opt "test"`);
            }
        }

        if (apps.includes(App.InternetIdentityTest)) {
            DFX.DEPLOY_II();
            var response = DFX.INIT_SALT();

            if (response !== "()") {
                continue;
            }

            dfx.iit.id = DFX.GET_CANISTER_ID("internet_identity_test");
            console.debug(">> ", dfx.iit.id);

            dfx.iit.actor = await getActor(dfx.iit.id, dfx.user.identity, iitIdl);

            imConfigurationArguments.push(`ii_canister_id = opt principal "${dfx.iit.id}"`);
        }

        if (apps.includes(App.IdentityManagerReplica)) {
            DFX.DEPLOY("identity_manager_replica");
            var response = DFX.CONFIGURE_REPLICA(dfx.im.id);

            if (response !== "()") {
                continue;
            }

            dfx.imr.id = DFX.GET_CANISTER_ID("identity_manager_replica");
            console.debug(">> ", dfx.imr.id);

            imConfigurationArguments.push(`heartbeat = opt 1`);
            imConfigurationArguments.push(`backup_canister_id = opt "${dfx.imr.id}"`);
        }

        DFX.CONFIGURE_IM(imConfigurationArguments.join("; "));

        return dfx;
    }

    DFX.STOP();
    process.exit(1);
};

export const getIdentity = (seed: string): Ed25519KeyIdentity => {
    let seedEncoded = new TextEncoder().encode(seed);
    return Ed25519KeyIdentity.generate(seedEncoded);
};

export const getActor = async (
    imCanisterId: string,
    identity: Identity,
    idl: IDL.InterfaceFactory
): Promise<Record<string, ActorMethod>> => {
    const agent: HttpAgent = new HttpAgent({ host: localhost, identity: identity });
    await agent.fetchRootKey();
    return await Actor.createActor(idl, { agent, canisterId: imCanisterId });
};