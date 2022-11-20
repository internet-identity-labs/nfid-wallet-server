import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export interface Conf { 'ledger_canister_id' : Principal }
export type Memo = bigint;
export interface Tokens { 'e8s' : bigint }
export interface TransferArgs {
  'to_principal' : Principal,
  'to_subaccount' : [] | [Array<number>],
  'amount' : Tokens,
}
export type TransferResult = { 'Ok' : Memo } |
  { 'Err' : string };
export interface _SERVICE {
  'sub' : ActorMethod<[string, number], string>,
  'transfer' : ActorMethod<[TransferArgs], TransferResult>,
}
