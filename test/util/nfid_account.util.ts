import * as Agent from "@icp-sdk/core/agent";
import {expect} from "chai";
import {Ed25519KeyIdentity} from "@icp-sdk/core/identity";
import {Dfx} from "../type/dfx";
import {BoolHttpResponse, HTTPAccountRequest, _SERVICE as IdentityManagerType} from "../idl/identity_manager";
import {idlFactory as imIdl} from "../idl/identity_manager_idl";
import {getIdentity, getTypedActor} from "./deployment.util";

export const buildIdentity = (label: string) => getIdentity(label.padEnd(32, "0"));

export interface NfidAccountFixture {
    emailIdentity: Ed25519KeyIdentity;
    emailActor: Agent.ActorSubclass<IdentityManagerType>;
    passkeyIdentity?: Ed25519KeyIdentity;
    passkeyActor?: Agent.ActorSubclass<IdentityManagerType>;
    recoveryIdentity?: Ed25519KeyIdentity;
    recoveryActor?: Agent.ActorSubclass<IdentityManagerType>;
}

export async function setupNfidAccount(
    dfx: Dfx,
    label: string,
    options: { email: string; withPasskey: boolean; withRecovery: boolean }
): Promise<NfidAccountFixture> {
    const emailIdentity = buildIdentity(`ap-email-${label}`);
    const emailPrincipal = emailIdentity.getPrincipal().toText();
    const emailActor = await getTypedActor<IdentityManagerType>(dfx.im.id, emailIdentity, imIdl);

    const validation = await dfx.im.actor.add_email_and_principal_for_create_account_validation(
        options.email, emailPrincipal, 25n
    ) as BoolHttpResponse;
    expect(validation.status_code).eq(200);

    const accountRequest: HTTPAccountRequest = {
        access_point: [{
            icon: "Icon",
            device: "Email device",
            pub_key: emailPrincipal,
            browser: "Browser",
            device_type: {Email: null},
            credential_id: []
        }],
        wallet: [{NFID: null}],
        anchor: 0n,
        email: [options.email],
        name: [],
        challenge_attempt: []
    };
    const account = await emailActor.create_account(accountRequest);
    expect(account.status_code).eq(200);

    const fixture: NfidAccountFixture = {emailIdentity, emailActor};

    if (options.withPasskey) {
        fixture.passkeyIdentity = buildIdentity(`ap-passkey-${label}`);
        fixture.passkeyActor = await getTypedActor<IdentityManagerType>(dfx.im.id, fixture.passkeyIdentity, imIdl);
        const passkeyResponse = await emailActor.create_access_point({
            icon: "Icon",
            device: "Passkey",
            pub_key: fixture.passkeyIdentity.getPrincipal().toText(),
            browser: "",
            device_type: {Passkey: null},
            credential_id: ["pk-cred-" + label]
        });
        expect(passkeyResponse.status_code).eq(200);
    }

    if (options.withRecovery) {
        fixture.recoveryIdentity = buildIdentity(`ap-recovery-${label}`);
        fixture.recoveryActor = await getTypedActor<IdentityManagerType>(dfx.im.id, fixture.recoveryIdentity, imIdl);
        const recoveryResponse = await emailActor.create_access_point({
            icon: "document",
            device: "seedphrase",
            pub_key: fixture.recoveryIdentity.getPrincipal().toText(),
            browser: "",
            device_type: {Recovery: null},
            credential_id: []
        });
        expect(recoveryResponse.status_code).eq(200);
    }

    return fixture;
}
