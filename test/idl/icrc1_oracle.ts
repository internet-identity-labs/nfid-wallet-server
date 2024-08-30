import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export type Category = { 'Sns' : null } |
    { 'Known' : null } |
    { 'Unknown' : null } |
    { 'ChainFusionTestnet' : null } |
    { 'ChainFusion' : null } |
    { 'Community' : null } |
    { 'Native' : null };
export interface Conf {
    'controllers' : [] | [Array<Principal>],
    'im_canister' : [] | [Principal],
}
export interface ICRC1 {
    'logo' : [] | [string],
    'name' : string,
    'ledger' : string,
    'category' : Category,
    'index' : [] | [string],
    'symbol' : string,
}
export interface ICRC1Request {
    'logo' : [] | [string],
    'name' : string,
    'ledger' : string,
    'index' : [] | [string],
    'symbol' : string,
}
export interface _SERVICE {
    'get_all_icrc1_canisters' : ActorMethod<[], Array<ICRC1>>,
    'replace_icrc1_canisters' : ActorMethod<[Array<ICRC1>], Array<ICRC1>>,
    'store_new_icrc1_canisters' : ActorMethod<[Array<ICRC1>], Array<ICRC1>>,
    'store_icrc1_canister' : ActorMethod<[ICRC1Request], undefined>,
    'sync_controllers' : ActorMethod<[], Array<string>>,
}
export declare const idlFactory: IDL.InterfaceFactory;
