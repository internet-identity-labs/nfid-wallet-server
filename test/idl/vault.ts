import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export interface Account {
    'account_id' : number,
    'name' : string,
    'sub_account' : Array<number>,
}
export interface Conf { 'ledger_canister_id' : Principal }
export interface Group {
    'participants' : Array<Participant>,
    'name' : string,
    'accounts' : Array<Account>,
    'group_id' : bigint,
}
export type Memo = bigint;
export type Participant = {};
export interface Tokens { 'e8s' : bigint }
export interface TransferArgs {
    'to_principal' : Principal,
    'to_subaccount' : [] | [Array<number>],
    'amount' : Tokens,
}
export type TransferResult = { 'Ok' : Memo } |
    { 'Err' : string };
export interface _SERVICE {
    'register_group' : ActorMethod<[string], Group>,
    'sub' : ActorMethod<[string, bigint, number], string>,
}
