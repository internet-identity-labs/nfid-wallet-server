import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export interface Conf { 'im_canister' : [] | [string] }
export interface _SERVICE {
    'get_canisters_by_root' : ActorMethod<[string], Array<string>>,
    'store_icrc1_canister' : ActorMethod<[string], undefined>,
}
