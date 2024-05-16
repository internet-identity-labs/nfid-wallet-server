import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface Conf { 'im_canister' : [] | [string] }
export interface ICRC1 { 'ledger' : string, 'index' : [] | [string] }
export interface _SERVICE {
    'get_canisters_by_root' : ActorMethod<[string], Array<ICRC1>>,
    'remove_icrc1_canister' : ActorMethod<[string], undefined>,
    'store_icrc1_canister' : ActorMethod<[string, [] | [string]], undefined>,
}
export declare const idlFactory: IDL.InterfaceFactory;
