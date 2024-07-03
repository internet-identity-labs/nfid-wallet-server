import {Dfx} from "./type/dfx";
import {deploy, getActor, getIdentity} from "./util/deployment.util";
import {App} from "./constanst/app.enum";
import {expect} from "chai";
import {GetDelegationResponse} from "./idl/delegation_factory";
import {Delegation, DelegationChain, DelegationIdentity} from "@dfinity/identity";
import {DerEncodedPublicKey, Signature} from "@dfinity/agent";
import {Principal} from "@dfinity/principal";
import {fail} from "assert";
import {DFX} from "./constanst/dfx.const";
import {AccessPointRequest, HTTPAccountRequest, HTTPAccountResponse} from "./idl/identity_manager";
import {idlFactory as imIdl} from "./idl/identity_manager_idl";
import {idlFactory as dfIdl} from "./idl/delegation_factory_idl";

describe("Delegation Factory test", () => {
    var dfx: Dfx;
    var dfActor: any;
    const targets = [[Principal.fromText("74gpt-tiaaa-aaaak-aacaa-cai")]];
    const sessionPair = getIdentity("87654321876543218765432187654322")
    const pk = new Uint8Array(
        sessionPair.getPublicKey().toDer(),
    )
    before(async () => {
        dfx = await deploy({apps: [App.IdentityManager, App.DelegationFactory]});
    });

    after(() => {
        DFX.STOP();
    });

    it("Get delegation", async function () {
        const identity = getIdentity("87654321876543218765432187654311");
        const principal = identity.getPrincipal().toText();
        const dd: AccessPointRequest = {
            icon: "Icon",
            device: "Global",
            pub_key: principal,
            browser: "Browser",
            device_type: {
                Email: null,
            },
            credential_id: [],
        };
        var accountRequest: HTTPAccountRequest = {
            access_point: [dd],
            wallet: [{NFID: null}],
            anchor: 0n,
            email: ["test@test.test"],
        };
        const actor = await getActor(dfx.im.id, identity, imIdl);
        await dfx.im.actor.add_email_and_principal_for_create_account_validation("test@test.test", principal, 25);
        const accResponse: HTTPAccountResponse = (await actor.create_account(
            accountRequest
        )) as HTTPAccountResponse;
        const response = accResponse.data[0];
        expect(response.anchor).eq(100000000n);

        dfActor = await getActor(dfx.delegation_factory.id, identity, dfIdl);

        let prepareDelegationResponse
        try {
            prepareDelegationResponse = await dfActor.prepare_delegation(
                response.anchor,
                "nfid.one",
                pk,
                [],
                targets
            )
            fail("Salt is set")
        } catch (e) {
            expect(e.message).contains("Salt not set")
            await dfActor.init_salt();
            prepareDelegationResponse = await dfActor.prepare_delegation(
                response.anchor,
                "nfid.one",
                pk,
                [],
                targets
            )
        }

        expect(prepareDelegationResponse[0]).not.undefined
        expect(prepareDelegationResponse[1]).not.undefined

        const chain = await dfActor.get_delegation(
            response.anchor,
            "nfid.one",
            pk,
            prepareDelegationResponse[1],
            targets
        ).then((r: GetDelegationResponse) => {
            if ("signed_delegation" in r) {
                return DelegationChain.fromDelegations(
                    [
                        {
                            delegation: new Delegation(
                                new Uint8Array(r.signed_delegation.delegation.pubkey).buffer,
                                r.signed_delegation.delegation.expiration,
                                mapOptional(r.signed_delegation.delegation.targets),
                            ),
                            signature: new Uint8Array(r.signed_delegation.signature)
                                .buffer as Signature,
                        },
                    ],
                    new Uint8Array(prepareDelegationResponse[0]),
                )
            } else {
                throw new Error("No such delegation")
            }
        })

        const delegationIdentity = DelegationIdentity.fromDelegation(
            sessionPair,
            chain,
        )
        const principalNfid = await dfActor.get_principal(response.anchor, "nfid.one")

        expect(delegationIdentity.getPrincipal().toText()).eq(principalNfid.toText())

    })

    it("Get delegation - Unauthorized", async function () {
        try {
            let resp = await dfActor.prepare_delegation(
                100000002n,
                "nfid.one",
                pk,
                [],
                targets
            )
            await dfActor.get_delegation(
                100000002n,
                "nfid.one",
                pk,
                resp[1],
                targets
            )
        } catch (e) {
            expect(e.message).contains("Unauthorised")
        }
    })
})


export function mapOptional<T>(value: [T] | []): T | undefined {
    if (value.length) return value[0]
    return undefined
}