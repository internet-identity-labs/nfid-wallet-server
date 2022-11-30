import {ActorMethod, Identity} from "@dfinity/agent";
import { Ed25519KeyIdentity } from "@dfinity/identity";

export interface Dfx {
    root: string;
    user: {
        principal: string;
        identity: Ed25519KeyIdentity;
    }
    im?: {
        id: string;
        actor: Record<string, ActorMethod>;
    }
    imr?: {
        id: string;
    };
    iit?: {
        id: string;
        actor: Record<string, ActorMethod>;
        anchor: bigint;
    },
    vault?: {
        id: string;
        actor: Record<string, ActorMethod>;
        actor_member: Record<string, ActorMethod>;
        member: Identity;
    };
    ess?: {
        id: string;
        actor: Record<string, ActorMethod>;
    }
};

export interface Configuration {
    clean?: boolean,
    apps: string[]
}