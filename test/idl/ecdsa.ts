import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export interface KeyPair {
    'public_key' : string,
    'private_key_encrypted' : string,
}
export interface KeyPairResponse {
    'key_pair' : [] | [KeyPair],
    'princ' : string,
}
export interface _SERVICE {
    'add_kp' : ActorMethod<[KeyPair], undefined>,
    'get_kp' : ActorMethod<[], KeyPairResponse>,
    'get_pk_by_principal' : ActorMethod<[string], string>,
    'get_principal' : ActorMethod<[[] | [string]], [string, [] | [string]]>,
    'get_all_json' : ActorMethod<[number, number], string>,
    'sync_controllers' : ActorMethod<[], Array<string>>,
    'count' : ActorMethod<[], bigint>,
}
