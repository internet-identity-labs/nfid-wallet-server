import 'mocha';
import { expect } from 'chai';
import { Dfx } from './type/dfx';
import { App } from './constanst/app.enum';
import { deploy } from './util/deployment.util';
import { DFX } from './constanst/dfx.const';
import { CertifiedKeyPairResponse, KeyPair, KeyPairResponse } from './idl/ecdsa';
import { fail } from 'assert';
import { compare, HttpAgent, lookup_path, LookupResultFound } from '@dfinity/agent';
import { Principal } from '@dfinity/principal';
import { verifyCertification } from './util/cert_verification';
import * as crypto from 'crypto';

describe('ECDSA signer test', () => {
  describe('ECDSA tests', () => {
    var dfx: Dfx;

    before(async () => {
      dfx = await deploy({ apps: [App.ECDSASigner] });
    });

    it('verify controllers', async function () {
      try {
        await dfx.ic_signer.actor.get_kp_certified(dfx.user.identity.getPrincipal().toText());
        fail('Should unauthorised');
      } catch (e) {
        expect(e.message).contains('Unauthorised');
      }
      try {
        await dfx.ic_signer.actor.get_all_json(0, 10);
        fail('Should unauthorised');
      } catch (e) {
        expect(e.message).contains('Unauthorised');
        DFX.USE_TEST_ADMIN();
        DFX.ADD_CONTROLLER(dfx.user.identity.getPrincipal().toText(), 'signer_ic');
        DFX.ADD_CONTROLLER(dfx.ic_signer.id, 'signer_ic');
      }
      await dfx.ic_signer.actor.sync_controllers();
    });

    it('should return key pair', async function () {
      let kp: KeyPair = {
        private_key_encrypted: 'test_private',
        public_key: 'test_public',
      };
      let emptyResponse = (await dfx.ic_signer.actor.get_kp()) as KeyPairResponse;
      expect(emptyResponse.key_pair.length).eq(0);
      await dfx.ic_signer.actor.add_kp(kp);
      try {
        await dfx.ic_signer.actor.add_kp(kp);
      } catch (e) {
        expect(e.message.includes('Already registered'));
      }
      let response = (await dfx.ic_signer.actor.get_kp()) as KeyPairResponse;
      expect(response.key_pair[0].public_key).eq('test_public');
      expect(response.key_pair[0].private_key_encrypted).eq('test_private');
      expect(response.princ).eq(dfx.user.identity.getPrincipal().toText());
      let certifiedResponse = (await dfx.ic_signer.actor.get_kp_certified(
        dfx.user.identity.getPrincipal().toText()
      )) as CertifiedKeyPairResponse;
      await verifyCertifiedResponse(certifiedResponse, dfx);
      DFX.UPGRADE_FORCE('signer_eth');
      response = (await dfx.ic_signer.actor.get_kp()) as KeyPairResponse;
      let certifiedResponseAfterAll = (await dfx.ic_signer.actor.get_kp_certified(
        dfx.user.identity.getPrincipal().toText()
      )) as CertifiedKeyPairResponse;
      expect(certifiedResponse.response.princ).eq(certifiedResponseAfterAll.response.princ);
      await verifyCertifiedResponse(certifiedResponseAfterAll, dfx);
      expect(response.key_pair[0].public_key).eq('test_public');
      expect(response.key_pair[0].private_key_encrypted).eq('test_private');
    });

    it('should return public key', async function () {
      let response = (await dfx.ic_signer.actor.get_public_key(dfx.user.principal)) as string[];
      expect(response[0]).eq('test_public');
    });

    it('should backup', async function () {
      let count = await dfx.ic_signer.actor.count();
      expect(count).eq(1n);
      let json = await dfx.ic_signer.actor.get_all_json(0, 10);
      expect(json).contains('public_key');
      expect(json).contains('test_public');
      expect(json).contains('private_key');
      expect(json).contains('test_private');
      expect(json).contains('principal');
      expect(json).contains('created_date');
    });
  });
});

async function verifyCertifiedResponse(certifiedResponse: CertifiedKeyPairResponse, dfx) {
  const agent = new HttpAgent({ host: 'http://127.0.0.1:8000' });
  await agent.fetchRootKey();
  const tree = await verifyCertification({
    canisterId: Principal.fromText(dfx.ic_signer.id),
    encodedCertificate: new Uint8Array(certifiedResponse.certificate).buffer,
    encodedTree: new Uint8Array(certifiedResponse.witness).buffer,
    rootKey: agent.rootKey,
    maxCertificateTimeOffsetMs: 50000,
  });
  const treeHash = lookup_path([dfx.user.identity.getPrincipal().toText()], tree);
  if (!treeHash) {
    throw new Error('Response not found in tree');
  }
  const newOwnedString =
    certifiedResponse.response.key_pair[0].public_key +
    certifiedResponse.response.key_pair[0].private_key_encrypted;
  const sha256Result = crypto.createHash('sha256').update(newOwnedString).digest();
  const byteArray = new Uint8Array(sha256Result);
  if (!equal(byteArray, (treeHash as LookupResultFound).value as ArrayBuffer)) {
    throw new Error('Response hash does not match');
  }
}

function equal(a: ArrayBuffer, b: ArrayBuffer): boolean {
  return compare(a, b) === 0;
}
