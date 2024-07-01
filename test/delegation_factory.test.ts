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

        const getDelegationResponse = await dfActor.get_delegation(
            response.anchor,
            "nfid.one",
            pk,
            prepareDelegationResponse[1],
            targets
        ).then((r: GetDelegationResponse) => {
            if ("signed_delegation" in r) {
                expect(r.signed_delegation.delegation.targets[0].length).eq(1)
                expect(r.signed_delegation.delegation.targets[0][0].toText()).eq("74gpt-tiaaa-aaaak-aacaa-cai")
                return {
                    delegation: {
                        expiration: r.signed_delegation.delegation.expiration,
                        pubkey: r.signed_delegation.delegation.pubkey,
                        targets: r.signed_delegation.delegation.targets,
                    },
                    signature: r.signed_delegation.signature,
                }
            } else {
                throw new Error("No such delegation")
            }
        })

        const chain = DelegationChain.fromDelegations(
            [
                {
                    delegation: new Delegation(
                        new Uint8Array(getDelegationResponse.delegation.pubkey).buffer,
                        getDelegationResponse.delegation.expiration,
                        getDelegationResponse.delegation.targets
                            .map((t) => t.map((p) => p.toText())),
                    ),
                    signature: new Uint8Array(getDelegationResponse.signature)
                        .buffer as Signature,
                },
            ],
            new Uint8Array(sessionPair.getPublicKey().toDer()).buffer as DerEncodedPublicKey,
        )

        const delegationIdentity = DelegationIdentity.fromDelegation(
            sessionPair,
            chain,
        )

        expect("st6dr-wqxcv-tret2-xxknz-it4bo-zp76f-ui335-nxzd4-peh3r-wzrsi-5ae").eq(
            delegationIdentity.getPrincipal().toText()
        )
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