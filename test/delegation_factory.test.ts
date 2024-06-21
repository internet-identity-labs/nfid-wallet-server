import {Dfx} from "./type/dfx";
import {deploy, getIdentity} from "./util/deployment.util";
import {App} from "./constanst/app.enum";
import {DFX} from "./constanst/dfx.const";
import {expect} from "chai";
import {GetDelegationResponse} from "./idl/delegation_factory";
import {Delegation, DelegationChain, DelegationIdentity} from "@dfinity/identity";
import {DerEncodedPublicKey, Signature} from "@dfinity/agent";

describe("Delegation Factory test", () => {
    var dfx: Dfx;

    before(async () => {
        dfx = await deploy({apps: [App.DelegationFactory]});
    });

    after(() => {
        DFX.STOP();
    });

    it("Get delegation", async function () {
        const sessionPair = getIdentity("87654321876543218765432187654311")
        const pk = new Uint8Array(
            sessionPair.getPublicKey().toDer(),
        )
        const prepareDelegationResponse = await dfx.delegation_factory.actor.prepare_delegation(
            BigInt(10001),
            "nfid.one",
            pk,
            []
        )
        expect(prepareDelegationResponse[0]).not.undefined
        expect(prepareDelegationResponse[1]).not.undefined

        const getDelegationResponse = await dfx.delegation_factory.actor.get_delegation(
            BigInt(10001),
            "nfid.one",
            pk,
            prepareDelegationResponse[1]
        ).then((r: GetDelegationResponse) => {
            if ("signed_delegation" in r) {
                return {
                    delegation: {
                        expiration: r.signed_delegation.delegation.expiration,
                        pubkey: r.signed_delegation.delegation.pubkey,
                        targets: [],
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
                        getDelegationResponse.delegation.targets,
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