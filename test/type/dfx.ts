import {ActorMethod, Identity} from "@dfinity/agent";
import { Ed25519KeyIdentity } from "@dfinity/identity";
import * as Agent from "@dfinity/agent"
import {_SERVICE as IdentityManager} from "../idl/identity_manager";
import {_SERVICE as InternetIdentityTest} from "../idl/internet_identity_test";

export interface Dfx {
    root: string;
    user: {
        principal: string;
        identity: Ed25519KeyIdentity;
    }
    im?: {
        id: string;
        actor: Agent.ActorSubclass<IdentityManager>;
    }
    imr?: {
        id: string;
    };
    iit?: {
        id: string;
        actor: Agent.ActorSubclass<InternetIdentityTest>;
        anchor: bigint;
    },
    vault?: {
        id: string;
        admin_actor: Record<string, ActorMethod>;
        actor_member_1: Record<string, ActorMethod>;
        actor_member_2: Record<string, ActorMethod>;
        member_1: Identity;
        member_2: Identity;
    };
    ess?: {
        id: string;
        actor: Record<string, ActorMethod>;
    };
    eth_signer?: {
        id: string;
        actor: Record<string, ActorMethod>;
    };
    btc?: {
        id: string;
        actor: Record<string, ActorMethod>;
    };
    icrc1?: {
        id: string;
        actor: Record<string, ActorMethod>;
    };
    icrc1_oracle?: {
        id: string;
        actor: Record<string, ActorMethod>;
    };
    delegation_factory?: {
        id: string;
        actor: Record<string, ActorMethod>;
    };
    nfid_storage?: {
        id: string;
        actor: Record<string, ActorMethod>;
    };
};

export interface Configuration {
    clean?: boolean,
    apps: string[]
}