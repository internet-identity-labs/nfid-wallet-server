import { ActorMethod } from "@dfinity/agent";

export interface Configuration {
    rootIdentity: string;
    principal: string;
    actor: Record<string, ActorMethod>;
};