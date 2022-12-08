import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export interface Approve {
    'status' : State,
    'signer' : string,
    'created_date' : bigint,
}
export interface Conf { 'ledger_canister_id' : Principal }
export type Currency = { 'ICP' : null };
export interface Policy { 'id' : bigint, 'policy_type' : PolicyType }
export interface PolicyRegisterRequest {
    'vault_id' : bigint,
    'policy_type' : PolicyType,
}
export type PolicyType = { 'threshold_policy' : ThresholdPolicy };
export type State = { 'REJECTED' : null } |
    { 'PENDING' : null } |
    { 'APPROVED' : null };
export interface ThresholdPolicy {
    'member_threshold' : number,
    'amount_threshold' : bigint,
    'wallet_ids' : [] | [Array<bigint>],
    'currency' : Currency,
}
export interface Tokens { 'e8s' : bigint }
export interface Transaction {
    'id' : bigint,
    'to' : string,
    'member_threshold' : number,
    'block_index' : [] | [bigint],
    'amount_threshold' : bigint,
    'state' : State,
    'approves' : Array<Approve>,
    'currency' : Currency,
    'amount' : Tokens,
    'created_date' : bigint,
    'modified_date' : bigint,
    'wallet_id' : bigint,
    'policy_id' : bigint,
}
export interface Vault {
    'id' : bigint,
    'members' : Array<VaultMember>,
    'name' : string,
    'wallets' : Array<bigint>,
    'policies' : Array<bigint>,
}
export interface VaultMember {
    'user_uuid' : string,
    'name' : [] | [string],
    'role' : VaultRole,
}
export interface VaultMemberRequest {
    'name' : [] | [string],
    'role' : VaultRole,
    'vault_id' : bigint,
    'address' : string,
}
export interface VaultRegisterRequest { 'name' : string }
export type VaultRole = { 'Member' : null } |
    { 'Admin' : null };
export interface Wallet {
    'id' : bigint,
    'name' : [] | [string],
    'vaults' : Array<bigint>,
}
export interface WalletRegisterRequest {
    'name' : [] | [string],
    'vault_id' : bigint,
}
export interface _SERVICE {
    'add_vault_member' : ActorMethod<[VaultMemberRequest], Vault>,
    'approve_transaction' : ActorMethod<[bigint, State], Transaction>,
    'get_policies' : ActorMethod<[bigint], Array<Policy>>,
    'get_transactions' : ActorMethod<[], Array<Transaction>>,
    'get_vault_members' : ActorMethod<[bigint], Array<VaultMember>>,
    'get_vaults' : ActorMethod<[], Array<Vault>>,
    'get_wallets' : ActorMethod<[bigint], Array<Wallet>>,
    'register_policy' : ActorMethod<[PolicyRegisterRequest], Policy>,
    'register_transaction' : ActorMethod<[Tokens, string, bigint], Transaction>,
    'register_vault' : ActorMethod<[VaultRegisterRequest], Vault>,
    'register_wallet' : ActorMethod<[WalletRegisterRequest], Wallet>,
    'sub' : ActorMethod<[bigint], string>,
}
