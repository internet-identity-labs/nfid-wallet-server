import * as Agent from '@dfinity/agent';

import { idlFactory as icrc1IDL } from './idl/icrc1';
import { _SERVICE as ICRC1ServiceIDL } from './idl/icrc1.idl';
import { HttpAgent } from '@dfinity/agent';

export const agentBaseConfig = { host: 'https://ic0.app' };

export async function getMetadata(ledger: string) {
  const actor = Agent.Actor.createActor<ICRC1ServiceIDL>(icrc1IDL, {
    canisterId: ledger,
    agent: new HttpAgent({ ...agentBaseConfig }),
  });
  const metadata = await actor.icrc1_metadata();
  let name = '';
  let symbol = '';
  let logo: string | undefined = undefined;
  let decimals = 0;
  let fee = BigInt(0);

  for (let i = 0; i < metadata.length; i++) {
    const data = metadata[i];
    if (data[0] === 'icrc1:name') {
      const val = data[1] as { Text: string };
      name = val.Text;
    } else if (data[0] === 'icrc1:symbol') {
      const val = data[1] as { Text: string };
      symbol = val.Text;
    } else if (data[0] === 'icrc1:decimals') {
      const val = data[1] as { Nat: bigint };
      decimals = Number(val.Nat);
    } else if (data[0] === 'icrc1:fee') {
      const val = data[1] as { Nat: bigint };
      fee = val.Nat;
    } else if (data[0] === 'icrc1:logo') {
      const val = data[1] as { Text: string };
      logo = val.Text;
    }
  }

  return {
    name,
    symbol,
    logo,
    decimals,
    fee,
    canister: ledger,
  };
}
