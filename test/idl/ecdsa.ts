import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export interface KeyPair {
  public_key: string;
  private_key_encrypted: string;
}
export interface KeyPairResponse {
  key_pair: [] | [KeyPair];
  princ: string;
}
export interface CertifiedKeyPairResponse {
  certificate: Uint8Array | number[];
  witness: Uint8Array | number[];
  response: KeyPairResponse;
}
export interface PublicKeyReply {
  public_key: Uint8Array;
}
export type Result = { Ok: SignatureReply } | { Err: string };
export type Result_1 = { Ok: PublicKeyReply } | { Err: string };
export interface SignatureReply {
  signature: Uint8Array;
}
export interface _SERVICE {
  add_kp: ActorMethod<[KeyPair], undefined>;
  count: ActorMethod<[], bigint>;
  get_all_json: ActorMethod<[number, number], string>;
  get_kp: ActorMethod<[], KeyPairResponse>;
  get_public_key: ActorMethod<[string], [] | [string]>;
  get_principal: ActorMethod<[[] | [string]], [string, [] | [string]]>;
  get_signature: ActorMethod<[string], Result>;
  prepare_signature: ActorMethod<[Uint8Array], string>;
  public_key: ActorMethod<[], Result_1>;
  sign: ActorMethod<[Uint8Array], Result>;
  sync_controllers: ActorMethod<[], Array<string>>;
  get_kp_certified: ActorMethod<[string], CertifiedKeyPairResponse>;
}
