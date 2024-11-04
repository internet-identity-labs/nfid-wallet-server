import * as Agent from "@dfinity/agent";
import {Actor, ActorMethod, HttpAgent, Identity} from "@dfinity/agent";
import {Ed25519KeyIdentity} from "@dfinity/identity";
import {Dfx} from "../type/dfx";
import {idlFactory as imIdl} from "../idl/identity_manager_idl";
import {idlFactory as vaultIdl} from "../idl/vault_idl";
import {idlFactory as icrc1Idl} from "../idl/icrc1_registry_idl";
import {idlFactory as icrcOracle1Idl} from "../idl/icrc1_oracle_idl";
import {idlFactory as iitIdl} from "../idl/internet_identity_test_idl";
import {idlFactory as esdsaIdl} from "../idl/ecdsa_idl";
import {idlFactory as delegationFactoryIDL} from "../idl/delegation_factory_idl";
import {idlFactory as nfidStorageIDL} from "../idl/nfid_storage_idl";
import {idlFactory as swapTrsStorageIDL} from "../idl/swap_trs_storage_idl";
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
    var imConfigurationArguments = new Set<string>;
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
        ic_signer: {
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
        icrc1: {
            id: null,
            actor: null,
        },
        icrc1_oracle: {
            id: null,
            actor: null,
        },
        delegation_factory: {
            id: null,
            actor: null,
        },
        nfid_storage: {
            id: null,
            actor: null,
        },
        swap_trs_storage: {
            id: null,
            actor: null,
        },
    };

    while (++i <= 5) {
        dfx.user.identity = getIdentity("87654321876543218765432187654321");
        dfx.user.principal = dfx.user.identity.getPrincipal().toString();

        if (clean) {            
            DFX.CREATE_TEST_PERSON();
            DFX.USE_TEST_ADMIN();
        }

        dfx.root = DFX.GET_PRINCIPAL();

        if (apps.includes(App.IdentityManager)) {
            if (clean) {
                DFX.DEPLOY_SPECIFIED("identity_manager", "74gpt-tiaaa-aaaak-aacaa-cai" );
            } else {
                DFX.UPGRADE_FORCE("identity_manager");
            }
            imConfigurationArguments.add(`operator = opt principal "${dfx.user.principal}"`);
            imConfigurationArguments.add(`lambda = opt principal "${dfx.user.principal}"`);
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
                imConfigurationArguments.add(`env = opt "test"`);
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

            imConfigurationArguments.add(`ii_canister_id = opt principal "${dfx.iit.id}"`);
        }

        if (apps.includes(App.ICRC1Registry)) {
            DFX.USE_TEST_ADMIN();
            DFX.DEPLOY_WITH_ARGUMENT("icrc1_registry", "(record { })");
            dfx.icrc1.id = DFX.GET_CANISTER_ID("icrc1_registry");
            dfx.icrc1.actor = await getActor(dfx.icrc1.id, dfx.user.identity, icrc1Idl);
        }

        if (apps.includes(App.ICRC1Oracle)) {
            DFX.USE_TEST_ADMIN();
            DFX.DEPLOY_WITH_ARGUMENT("icrc1_oracle", "(opt record { })");
            dfx.icrc1_oracle.id = DFX.GET_CANISTER_ID("icrc1_oracle");
            dfx.icrc1_oracle.actor = await getActor(dfx.icrc1_oracle.id, dfx.user.identity, icrcOracle1Idl);
            DFX.ADD_CONTROLLER(dfx.user.identity.getPrincipal().toText(), "icrc1_oracle");
        }

        if (apps.includes(App.IdentityManager)) {
            DFX.CONFIGURE_IM(Array.from(imConfigurationArguments).join("; "));
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

            dfx.ic_signer.id = DFX.GET_CANISTER_ID("signer_ic");
            console.log(">> ", dfx.ic_signer.id);

            dfx.ic_signer.actor = await getActor(dfx.ic_signer.id, dfx.user.identity, esdsaIdl);
            return dfx;
        }
        if (apps.includes(App.DelegationFactory)) {
            execute(`dfx deploy delegation_factory --mode reinstall -y --argument '(opt record { im_canister = principal "${dfx.im.id}" })'`)

            dfx.delegation_factory.id = DFX.GET_CANISTER_ID("delegation_factory");
            console.log(">> ", dfx.delegation_factory.id);

            dfx.delegation_factory.actor = await getActor(dfx.delegation_factory.id, dfx.user.identity, delegationFactoryIDL);
            return dfx;
        }
        if (apps.includes(App.NFIDStorage)) {
            execute(`dfx deploy nfid_storage --mode reinstall -y --argument '(opt record { im_canister = principal "${dfx.im.id}" })'`)

            dfx.nfid_storage.id = DFX.GET_CANISTER_ID("nfid_storage");
            console.log(">> ", dfx.nfid_storage.id);

            dfx.nfid_storage.actor = await getActor(dfx.nfid_storage.id, dfx.user.identity, nfidStorageIDL);
            return dfx;
        }
        if (apps.includes(App.SwapTrsStorage)) {
            execute(`dfx deploy swap_trs_storage --mode reinstall -y --argument '(opt record { im_canister = principal "${dfx.im.id}" })'`)

            dfx.swap_trs_storage.id = DFX.GET_CANISTER_ID("swap_trs_storage");
            console.log(">> ", dfx.swap_trs_storage.id);

            dfx.swap_trs_storage.actor = await getActor(dfx.swap_trs_storage.id, dfx.user.identity, swapTrsStorageIDL);
            return dfx;
        }

        DFX.CONFIGURE_IM(Array.from(imConfigurationArguments).join("; "));

        return dfx;
    }

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
