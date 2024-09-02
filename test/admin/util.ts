import {Category} from "../idl/icrc1_oracle";
import {CategoryCSV} from "./types";
import {Actor, ActorMethod, HttpAgent, Identity} from "@dfinity/agent";
import {IDL} from "@dfinity/candid";


export function mapCategory(category: Category): CategoryCSV {
    if (hasOwnProperty(category, "Sns")) {
        return CategoryCSV.Sns
    }
    if (hasOwnProperty(category, "Known")) {
        return CategoryCSV.Known
    }
    if (hasOwnProperty(category, "Unknown")) {
        return CategoryCSV.Unknown
    }
    if (hasOwnProperty(category, "ChainFusionTestnet")) {
        return CategoryCSV.ChainFusionTestnet
    }
    if (hasOwnProperty(category, "ChainFusion")) {
        return CategoryCSV.ChainFusion
    }
    if (hasOwnProperty(category, "Community")) {
        return CategoryCSV.Community
    }
    if (hasOwnProperty(category, "Native")) {
        return CategoryCSV.Native
    }
    throw new Error("Unknown category")
}

export function mapCategoryCSVToCategory(categoryCSV: string): Category {
    switch (categoryCSV) {
        case CategoryCSV.Sns:
            return {Sns: null};
        case CategoryCSV.Known:
            return {Known: null};
        case CategoryCSV.Native:
            return {Native: null};
        case CategoryCSV.Unknown:
            return {Unknown: null};
        case CategoryCSV.ChainFusionTestnet:
            return {ChainFusionTestnet: null};
        case CategoryCSV.ChainFusion:
            return {ChainFusion: null};
        case CategoryCSV.Community:
            return {Community: null};
        default:
            throw new Error(`Unknown category: ${JSON.stringify(categoryCSV)}`);
    }
}

export function hasOwnProperty<
    X extends Record<string, unknown>,
    Y extends PropertyKey,
>(obj: X, prop: Y): obj is X & Record<Y, unknown> {
    return Object.prototype.hasOwnProperty.call(obj, prop)
}

export function getActor(
    imCanisterId: string,
    identity: Identity,
    idl: IDL.InterfaceFactory
): Record<string, ActorMethod> {
    const agent: HttpAgent = new HttpAgent({host: "https://ic0.app", identity: identity});
    return Actor.createActor(idl, {agent, canisterId: imCanisterId});
};