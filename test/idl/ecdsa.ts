import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export interface PublicKeyReply { 'public_key' : Array<number> }
export interface RawSignature {
    'r' : Array<bigint>,
    's' : Array<bigint>,
    'v' : bigint,
}
export type Result = { 'Ok' : PublicKeyReply } |
    { 'Err' : string };
export type Result_1 = { 'Ok' : SignatureReply } |
    { 'Err' : string };
export interface SignatureReply {
    'signature' : Array<number>,
    'hex_signature' : string,
    'raw_signature' : RawSignature,
}
export interface _SERVICE {
    'public_key' : ActorMethod<[], Result>,
    'sign' : ActorMethod<[Array<number>], Result_1>,
}
