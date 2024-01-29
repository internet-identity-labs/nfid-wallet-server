import * as Agent from "@dfinity/agent";
import {Actor, ActorMethod, HttpAgent, Identity} from "@dfinity/agent";
import {Ed25519KeyIdentity} from "@dfinity/identity";
import {Dfx} from "../type/dfx";
import {idlFactory as imIdl} from "../idl/identity_manager_idl";
import {idlFactory as vaultIdl} from "../idl/vault_idl";
import {idlFactory as icrc1Idl} from "../idl/icrc1_registry_idl";
import {idlFactory as iitIdl} from "../idl/internet_identity_test_idl";
import {idlFactory as essIdl} from "../idl/eth_secret_storage_idl";
import {idlFactory as esdsaIdl} from "../idl/ecdsa_idl";
import {TextEncoder} from "util";
import {App} from "../constanst/app.enum";
import {IDL} from "@dfinity/candid";
import {DFX} from "../constanst/dfx.const";
import {execute} from "./call.util";
import {_SERVICE as IdentityManagerType} from "../idl/identity_manager"
import {_SERVICE as InternetIdentityTest} from "../idl/internet_identity_test"

const localhost: string = "http://127.0.0.1:8000";

export const deploy = async ({clean = true, apps}: { clean?: boolean, apps: App[] }): Promise<Dfx> => {
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
        ess: {
            id: null,
            actor: null,
        },
        eth_signer: {
            id: null,
            actor: null,
        },
        vault: {
            id: null,
            admin_actor: null,
            actor_member_1: null,
            actor_member_2: null,
            member_1: null,
            member_2: null
        },
        btc: {
            id: null,
            actor: null,
        },
        icrc1: {
            id: null,
            actor: null,
        },
    };

    while (++i <= 5) {
        dfx.user.identity = getIdentity("87654321876543218765432187654321");
        dfx.user.principal = dfx.user.identity.getPrincipal().toString();

        if (clean) {
            DFX.STOP();
            DFX.REMOVE_DFX_FOLDER();
            DFX.CREATE_TEST_PERSON();
            DFX.USE_TEST_ADMIN();
        }

        dfx.root = DFX.GET_PRINCIPAL();

        if (clean) {
            DFX.INIT();
        }

        if (apps.includes(App.EthSecretStorage)) {
            if (clean) {
                DFX.DEPLOY("eth_secret_storage");
            } else {
                DFX.UPGRADE_FORCE("eth_secret_storage");
            }

            var response = DFX.INIT_ESS();
            console.debug(">> ", response);

            if (response !== "()") {
                continue;
            }

            dfx.ess.id = DFX.GET_CANISTER_ID("eth_secret_storage");
            console.debug(">> ", dfx.ess.id);

            dfx.ess.actor = await getActor(dfx.ess.id, dfx.user.identity, essIdl);
            return dfx;
        }

        if (apps.includes(App.IdentityManager)) {
            if (clean) {
                DFX.DEPLOY("identity_manager");
            } else {
                DFX.UPGRADE_FORCE("identity_manager");
            }
            var response = DFX.CONFIGURE();

            if (response !== "()") {
                continue;
            }

            dfx.im.id = DFX.GET_CANISTER_ID("identity_manager");
            console.debug(">> ", dfx.im.id);

            dfx.im.actor = await getTypedActor<IdentityManagerType>(dfx.im.id, dfx.user.identity, imIdl);

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

            dfx.iit.actor = await getTypedActor<InternetIdentityTest>(dfx.iit.id, dfx.user.identity, iitIdl);

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

        if (apps.includes(App.ICRC1Registry)) {
            DFX.USE_TEST_ADMIN();
            DFX.DEPLOY_WITH_ARGUMENT("icrc1_registry", "(record { })");
            dfx.icrc1.id = DFX.GET_CANISTER_ID("icrc1_registry");
            dfx.icrc1.actor = await getActor(dfx.icrc1.id, dfx.user.identity, icrc1Idl);
        }

        if (apps.includes(App.IdentityManager)) {
            DFX.CONFIGURE_IM(imConfigurationArguments.join("; "));
        }
        if (apps.includes(App.Vault)) {
            DFX.USE_TEST_ADMIN();
            await console.log(execute(`./test/resource/ledger.sh`))
            await console.log(execute(`./test/resource/vault.sh`))

            dfx.vault.id = DFX.GET_CANISTER_ID("vault");
            console.log(">> ", dfx.vault.id);

            dfx.vault.admin_actor = await getActor(dfx.vault.id, dfx.user.identity, vaultIdl);
            dfx.vault.member_1 = Ed25519KeyIdentity.generate();
            dfx.vault.member_2 = Ed25519KeyIdentity.generate();
            dfx.vault.actor_member_1 = await getActor(dfx.vault.id, dfx.vault.member_1, vaultIdl);
            dfx.vault.actor_member_2 = await getActor(dfx.vault.id, dfx.vault.member_2, vaultIdl);
            return dfx;
        }
        if (apps.includes(App.ECDSASigner)) {
            DFX.DEPLOY_ECDSA();

            dfx.eth_signer.id = DFX.GET_CANISTER_ID("signer_eth");
            console.log(">> ", dfx.eth_signer.id);

            dfx.eth_signer.actor = await getActor(dfx.eth_signer.id, dfx.user.identity, esdsaIdl);
            return dfx;
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
    const agent: HttpAgent = new HttpAgent({host: localhost, identity: identity});
    await agent.fetchRootKey();
    return await Actor.createActor(idl, {agent, canisterId: imCanisterId});
};

export async function getTypedActor<T>(
    imCanisterId: string,
    identity: Identity,
    idl: IDL.InterfaceFactory
): Promise<Agent.ActorSubclass<T>> {
    const agent: HttpAgent = new HttpAgent({host: localhost, identity: identity});
    await agent.fetchRootKey();
    return Actor.createActor(idl, {agent, canisterId: imCanisterId});
};
