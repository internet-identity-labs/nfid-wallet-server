import 'mocha';
import { expect } from 'chai';
import { Dfx } from './type/dfx';
import { App } from './constanst/app.enum';
import { deploy, getActor, getIdentity, getTypedActor } from './util/deployment.util';
import {
  AccessPointRemoveRequest,
  AccessPointRequest,
  BoolHttpResponse,
  CertifiedResponse,
  HTTPAccessPointResponse,
  HTTPAccountRequest,
  HTTPAccountResponse,
} from './idl/identity_manager';
import { DFX } from './constanst/dfx.const';
import { idlFactory as imIdl } from './idl/identity_manager_idl';
import { Ed25519KeyIdentity } from '@dfinity/identity';
import { fail } from 'assert';
import { _SERVICE as IdentityManagerType } from './idl/identity_manager';
import { verifyCertifiedResponse } from './util/cert_verification';
import { call } from './util/call.util';

describe('Access Point', () => {
  var dfx: Dfx;

  before(async () => {
    dfx = await deploy({ apps: [App.IdentityManager] });
  });

  it('should protect recovery phrase', async function () {
    const identity = getIdentity('87654321876543218765432187654311');
    const principal = identity.getPrincipal().toText();

    console.log(call(`dfx canister call identity_manager get_config`));

    const passKeyEmailRequest: AccessPointRequest = {
      icon: 'Icon',
      device: 'Global',
      pub_key: principal,
      browser: 'Browser',
      device_type: {
        Email: null,
      },
      credential_id: [],
    };
    var accountRequest: HTTPAccountRequest = {
      access_point: [passKeyEmailRequest],
      wallet: [{ NFID: null }],
      anchor: 0n,
      email: ['test@test.test'],
      name: [],
      challenge_attempt: [],
    };

    let response = (await dfx.im.actor.add_email_and_principal_for_create_account_validation(
      'test@test.test',
      principal,
      25n
    )) as BoolHttpResponse;
    expect(response.status_code).eq(200);

    const actor = await getActor(dfx.im.id, identity, imIdl);
    const acc = (await actor.create_account(accountRequest)) as HTTPAccountResponse;
    expect(acc.status_code).eq(200);
    const recoveryIdentity = Ed25519KeyIdentity.generate();
    var request: AccessPointRequest = {
      icon: '',
      device: '',
      pub_key: recoveryIdentity.getPrincipal().toText(),
      browser: '',
      device_type: {
        Recovery: null,
      },
      credential_id: [],
    };
    let ap = (await actor.create_access_point(request)) as HTTPAccessPointResponse;
    expect(ap.status_code).eq(200);

    let recoveryActor = await getActor(dfx.im.id, recoveryIdentity, imIdl);
    //verify certified response for passkey
    var certifiedResponse = (await recoveryActor.get_root_certified()) as CertifiedResponse;
    expect(certifiedResponse.witness.length > 0).eq(true);
    expect(certifiedResponse.response).eq(identity.getPrincipal().toText());

    var recoveryRemoveRequest: AccessPointRemoveRequest = {
      pub_key: recoveryIdentity.getPrincipal().toText(),
    };
    try {
      await actor.remove_access_point(recoveryRemoveRequest);
      fail('');
    } catch (e) {
      expect(e.message).contains('Recovery phrase is protected');
    }

    let pkIdentity = Ed25519KeyIdentity.generate();
    //get device back
    const passKeyRequest: AccessPointRequest = {
      icon: 'Icon',
      device: 'Global',
      pub_key: pkIdentity.getPrincipal().toText(),
      browser: 'Browser',
      device_type: {
        Passkey: null,
      },
      credential_id: [],
    };
    ap = (await actor.create_access_point(passKeyRequest)) as HTTPAccessPointResponse;
    expect(ap.status_code).eq(200);
    //verify certified response for recovery
    certifiedResponse = (await recoveryActor.get_root_certified()) as CertifiedResponse;
    expect(certifiedResponse.witness.length > 0).eq(true);
    expect(certifiedResponse.response).eq(identity.getPrincipal().toText());
    //verify that recovery phrase does not affect pass keys
    let removeFromPKActor = (await actor.remove_access_point({
      pub_key: pkIdentity.getPrincipal().toText(),
    })) as HTTPAccessPointResponse;
    expect(removeFromPKActor.status_code).eq(200);

    //verify that you can remove recovery from recovery
    let resp = (await recoveryActor.remove_access_point(
      recoveryRemoveRequest
    )) as HTTPAccessPointResponse;
    expect(resp.status_code).eq(200);

    //verify certified response removed for recovery
    try {
      await recoveryActor.get_root_certified();
      fail('Nope');
    } catch (e) {
      expect(e.message).contains('No such ap');
    }

    //verify that recovery principal removed from the index
    let resp2 = (await recoveryActor.remove_access_point({
      pub_key: identity.getPrincipal().toText(),
    })) as HTTPAccessPointResponse;
    expect(resp2.status_code).eq(404);

    //verify that we can remove root device (should not be a case for FE)
    let resp3 = (await actor.remove_access_point({
      pub_key: identity.getPrincipal().toText(),
    })) as HTTPAccessPointResponse;
    expect(resp3.status_code).eq(200);
  });

  it('should have device principal in certified map after app restarts.', async function () {
    const identity = getIdentity('87654321876543218765432187654377');
    const principal = identity.getPrincipal().toText();
    const email = 'test@test.test';

    const validationResponse =
      await dfx.im.actor.add_email_and_principal_for_create_account_validation(
        email,
        principal,
        25n
      );
    expect(validationResponse.status_code).eq(200);

    const accessPointRequest: AccessPointRequest = {
      icon: 'google',
      device: 'Google',
      pub_key: principal,
      browser: '',
      device_type: {
        Email: null,
      },
      credential_id: [],
    };

    var accountRequest: HTTPAccountRequest = {
      access_point: [accessPointRequest],
      wallet: [{ NFID: null }],
      anchor: 0n,
      email: [email],
      name: [],
      challenge_attempt: [],
    };

    const actor = await getTypedActor<IdentityManagerType>(dfx.im.id, identity, imIdl);
    const accountResponse = await actor.create_account(accountRequest);
    expect(accountResponse.status_code).eq(200);

    var rootCertifiedResponse = await actor.get_root_certified();
    expect(rootCertifiedResponse.witness.length > 0).eq(true);
    expect(rootCertifiedResponse.response).eq(principal);

    // Add recovery device.

    const recoveryIdentity = Ed25519KeyIdentity.generate();
    const recoveryPrincipal = recoveryIdentity.getPrincipal().toText();
    var request: AccessPointRequest = {
      icon: '',
      device: '',
      pub_key: recoveryPrincipal,
      browser: '',
      device_type: {
        Recovery: null,
      },
      credential_id: [],
    };
    let accessPointResponse = await actor.create_access_point(request);
    expect(accessPointResponse.status_code).eq(200);

    // Check existence of the device in device index.

    let recoveryActor = await getTypedActor<IdentityManagerType>(
      dfx.im.id,
      recoveryIdentity,
      imIdl
    );
    var { certificate, witness, response } = await recoveryActor.get_root_certified();
    expect(response).eq(identity.getPrincipal().toText());
    await verifyCertifiedResponse(certificate, witness, recoveryPrincipal, dfx.im.id, response);

    // Restart the app.

    dfx = await deploy({ clean: false, apps: [App.IdentityManager] });

    // It should have no device in device index after restart.

    var { certificate, witness, response } = await recoveryActor.get_root_certified();
    expect(response).eq(identity.getPrincipal().toText());

    try {
      await verifyCertifiedResponse(certificate, witness, recoveryPrincipal, dfx.im.id, response);
    } catch (e) {
      expect(e.message).to.equal(
        'Tree root hash did not match the certified data in the certificate.'
      );
    }

    // It should have it restored by the methods.

    const successStackResponse = await dfx.im.actor.save_temp_stack_to_rebuild_device_index();
    expect(successStackResponse).to.equal('The stack has been filled with data.');

    const errorStackResponse = await dfx.im.actor.save_temp_stack_to_rebuild_device_index();
    expect(errorStackResponse).to.equal('The stack is not empty. No action required.');

    const nonZeroRemainingAmount =
      await dfx.im.actor.get_remaining_size_after_rebuild_device_index_slice_from_temp_stack([1n]);
    expect(Number(nonZeroRemainingAmount)).to.be.gt(0);

    const zeroRemainingAmount =
      await dfx.im.actor.get_remaining_size_after_rebuild_device_index_slice_from_temp_stack([
        10000n,
      ]);
    expect(Number(zeroRemainingAmount)).to.equal(0);

    const zeroRemainingAmount2 =
      await dfx.im.actor.get_remaining_size_after_rebuild_device_index_slice_from_temp_stack([]);
    expect(Number(zeroRemainingAmount2)).to.equal(0);

    var { certificate, witness, response } = await recoveryActor.get_root_certified();
    expect(response).eq(identity.getPrincipal().toText());
    await verifyCertifiedResponse(certificate, witness, recoveryPrincipal, dfx.im.id, response);
  });
});
