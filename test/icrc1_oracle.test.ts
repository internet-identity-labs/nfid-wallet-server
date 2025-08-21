import { Dfx } from './type/dfx';
import { deploy, getActor, getIdentity } from './util/deployment.util';
import { App } from './constanst/app.enum';
import { expect } from 'chai';
import { ICRC1, NeuronData } from './idl/icrc1_oracle';
import { idlFactory } from './idl/icrc1_oracle_idl';
import { fail } from 'assert';
import { DFX } from './constanst/dfx.const';

describe('ICRC1 canister Oracle', () => {
  var dfx: Dfx;

  before(async () => {
    dfx = await deploy({ apps: [App.ICRC1Oracle] });
  });

  it('Set operator', async function () {
    const identity = getIdentity('87654321876543218765432187654311');
    const notAdmin = getIdentity('87654321876543218765432187654377');
    let dffActor = await getActor(dfx.icrc1_oracle.id, notAdmin, idlFactory);
    try {
      await dffActor.set_operator(notAdmin.getPrincipal());
      fail('Should throw an error');
    } catch (e) {
      expect(e.message).contains('Unauthorized');
    }
    DFX.ADD_CONTROLLER(identity.getPrincipal().toText(), 'icrc1_oracle');
    dfx.icrc1_oracle.actor = await getActor(dfx.icrc1_oracle.id, identity, idlFactory);
    await dfx.icrc1_oracle.actor.set_operator(identity.getPrincipal());
  });

  it('Store/retrieve canister id', async function () {
    let firstCanister: ICRC1 = {
      logo: ['logo'],
      name: 'name',
      ledger: 'ryjl3-tyaaa-aaaaa-aaaba-cai',
      index: ['irshc-3aaaa-aaaam-absla-cai'],
      symbol: 'symbol',
      category: { Spam: null },
      fee: BigInt(1),
      decimals: 1,
      root_canister_id: [],
      date_added: BigInt(Date.now()),
    };
    await dfx.icrc1_oracle.actor.store_icrc1_canister(firstCanister);
    let allCanisters = (await dfx.icrc1_oracle.actor.get_all_icrc1_canisters()) as Array<ICRC1>;
    expect(allCanisters.length).eq(1);
    expect(allCanisters[0].ledger).eq('ryjl3-tyaaa-aaaaa-aaaba-cai');
    expect(allCanisters[0].name).eq('name');
    expect(allCanisters[0].symbol).eq('symbol');
    expect(allCanisters[0].index).deep.eq(['irshc-3aaaa-aaaam-absla-cai']);
    expect(allCanisters[0].logo).deep.eq(['logo']);
    expect(allCanisters[0].category).deep.eq({ Community: null });

    const secondCanister: ICRC1 = {
      logo: ['logo2'],
      name: 'name2',
      ledger: 'irshc-3aaaa-aaaam-absla-cai',
      index: ['ryjl3-tyaaa-aaaaa-aaaba-cai'],
      symbol: 'symbol2',
      category: { Spam: null },
      fee: BigInt(1),
      decimals: 1,
      root_canister_id: [],
      date_added: BigInt(Date.now()),
    };
    const third: ICRC1 = {
      logo: ['logo3'],
      name: 'name3',
      ledger: 'c543j-2qaaa-aaaal-ac4dq-cai',
      index: ['ryjl3-tyaaa-aaaaa-aaaba-cai'],
      symbol: 'symbol3',
      category: { Spam: null },
      fee: BigInt(1),
      decimals: 1,
      root_canister_id: [],
      date_added: BigInt(Date.now()),
    };
    firstCanister = allCanisters[0];
    firstCanister.category = { Known: null };
    await dfx.icrc1_oracle.actor.replace_icrc1_canisters([firstCanister, secondCanister, third]);
    allCanisters = (await dfx.icrc1_oracle.actor.get_all_icrc1_canisters()) as Array<ICRC1>;
    expect(allCanisters.length).eq(3);
    expect(allCanisters.find((k) => k.ledger === firstCanister.ledger).category).deep.eq({
      Known: null,
    });
  });

  it('Count/getPaginated ICRC1', async function () {
    let canisters = (await dfx.icrc1_oracle.actor.count_icrc1_canisters()) as number;
    expect(canisters).eq(3n);
    let b = (await dfx.icrc1_oracle.actor.get_icrc1_paginated(0, 2)) as Array<ICRC1>;
    expect(b.length).eq(2);
    const offset = 2;
    let amountOfRequests = Math.ceil(Number(canisters) / offset);
    expect(amountOfRequests).eq(2);
    const all = await Promise.all(
      Array.from({ length: amountOfRequests }, (_, i) =>
        dfx.icrc1_oracle.actor.get_icrc1_paginated(i * offset, offset)
      )
    ).then((res) => res.flat());
    expect(all.length).eq(3);
  });

  it('Remove ICRC1', async function () {
    let allCanisters = (await dfx.icrc1_oracle.actor.get_all_icrc1_canisters()) as Array<ICRC1>;
    expect(allCanisters.length).eq(3);
    await dfx.icrc1_oracle.actor.remove_icrc1_canister(allCanisters[0].ledger);
    allCanisters = (await dfx.icrc1_oracle.actor.get_all_icrc1_canisters()) as Array<ICRC1>;
    expect(allCanisters.length).eq(2);
  });

  it('Replace neurons', async function () {
    let neurons: Array<NeuronData> = [
      {
        name: 'name',
        date_added: BigInt(Date.now()),
        ledger: 'ledger',
        neuron_id: 'neuron_id',
      },
      {
        name: 'name2',
        date_added: BigInt(Date.now()),
        ledger: 'ledger2',
        neuron_id: 'neuron_id2',
      },
    ];
    await dfx.icrc1_oracle.actor.replace_all_neurons(neurons);
    let allNeurons = (await dfx.icrc1_oracle.actor.get_all_neurons()) as Array<NeuronData>;
    expect(allNeurons.length).eq(2);
  });
});
