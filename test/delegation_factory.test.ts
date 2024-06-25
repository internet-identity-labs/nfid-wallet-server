import {Dfx} from "./type/dfx";
import {deploy, getIdentity} from "./util/deployment.util";
import {App} from "./constanst/app.enum";
import {DFX} from "./constanst/dfx.const";
import {expect} from "chai";
import {GetDelegationResponse} from "./idl/delegation_factory";
import {Delegation, DelegationChain, DelegationIdentity} from "@dfinity/identity";
import {DerEncodedPublicKey, Signature} from "@dfinity/agent";
import {Principal} from "@dfinity/principal";
import {fail} from "assert";

describe("Delegation Factory test", () => {
    var dfx: Dfx;

    before(async () => {
        dfx = await deploy({apps: [App.DelegationFactory]});
    });

    after(() => {
        DFX.STOP();
    });

    it("Get delegation", async function () {
        const targets = [[Principal.fromText("74gpt-tiaaa-aaaak-aacaa-cai")]];
        const sessionPair = getIdentity("87654321876543218765432187654311")
        const pk = new Uint8Array(
            sessionPair.getPublicKey().toDer(),
        )
        let prepareDelegationResponse
        try {
            prepareDelegationResponse = await dfx.delegation_factory.actor.prepare_delegation(
                BigInt(10002),
                "nfid.one",
                pk,
                [],
                targets
            )
            fail("Salt is set")
        } catch (e) {
            expect(e.message).contains("Salt not set")
            await dfx.delegation_factory.actor.init_salt();
            prepareDelegationResponse = await dfx.delegation_factory.actor.prepare_delegation(
                BigInt(10002),
                "nfid.one",
                pk,
                [],
                targets
            )
        }

        expect(prepareDelegationResponse[0]).not.undefined
        expect(prepareDelegationResponse[1]).not.undefined

        const getDelegationResponse = await dfx.delegation_factory.actor.get_delegation(
            BigInt(10002),
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

        expect("duwbk-wkvu3-p5ej3-z3w4g-j5opx-vfkl5-t6qc6-vkcah-2psnh-dbf6b-vae").eq(
            delegationIdentity.getPrincipal().toText()
        )
    })
})