import "mocha";
import {deploy} from "./util/deployment.util";
import {Dfx} from "./type/dfx";
import {App} from "./constanst/app.enum";
import {Policy, PolicyRegisterRequest} from "./idl/vault";
import {expect} from "chai";
import {principalToAddress} from "ictool"
import {DFX} from "./constanst/dfx.const";


let memberAddress: string;

describe("Policy", () => {
    var dfx: Dfx;

    before(async () => {
        dfx = await deploy(App.Vault);
        memberAddress = principalToAddress(
            dfx.vault.member.getPrincipal() as any,
            Array(32).fill(1));
        await dfx.vault.actor.register_vault({
            description: [],
            name: "vault1"
        })
        await dfx.vault.actor.register_vault({
            description: [],
            name: "vault2"
        })
        await dfx.vault.actor.store_member({
            address: memberAddress,
            name: ["MoyaLaskovayaSuchechka"],
            role: {'Member': null},
            vault_id: 1n,
            state: {'Active': null},
        });
    });

    after(() => {
        DFX.STOP();
    });

    it("register policy", async function () {

        let result1 = await dfx.vault.actor.register_policy({
            policy_type: {
                'threshold_policy': {
                    amount_threshold: 10n,
                    currency: {'ICP': null},
                    member_threshold: 2,
                    wallet_ids: []
                }
            },
            state: {'Active': null},
            vault_id: 1n
        } as PolicyRegisterRequest) as Policy
        verifyPolicy(result1, {
            state: {'Active': null},
            vault: 1n,
            created_date: 0n,
            id: 1n,
            modified_date: 0n,
            policy_type: {
                'threshold_policy': {
                    amount_threshold: 10n,
                    currency: {'ICP': null},
                    member_threshold: 2,
                    wallet_ids: []
                },

            }
        })
        expect(result1.created_date).eq(result1.modified_date);
        let result2 = await dfx.vault.actor.register_policy({
            policy_type: {
                'threshold_policy': {
                    amount_threshold: 2n,
                    currency: {'ICP': null},
                    member_threshold: 3,
                    wallet_ids: [[1n]]
                }
            },
            vault_id: 1n
        }) as Policy
        verifyPolicy(result2, {
            state: {'Active': null},
            vault: 1n,
            created_date: 0n,
            id: 2n,
            modified_date: 0n,
            policy_type: {
                'threshold_policy': {
                    amount_threshold: 2n,
                    currency: {'ICP': null},
                    member_threshold: 3,
                    wallet_ids: [[1n]]
                },
            }
        });
        let policies = await dfx.vault.actor.get_policies(1n) as [Policy]
        expect(policies.length).eq(2)
        let policy1 = policies.find(l => l.id === 1n)
        let policy2 = policies.find(l => l.id === 2n)
        verifyPolicy(policy1, result1)
        verifyPolicy(policy2, result2)
    });

    it("update policy", async function () {
        let policies = await dfx.vault.actor.get_policies(1n) as [Policy]
        let policy = policies.find(l => l.id === 1n)

        let updatePolicyRequest: Policy = {
            created_date: 1n,
            id: 1n,
            modified_date: 2n,
            policy_type: {
                'threshold_policy': {
                    amount_threshold: 2n,
                    currency: {'ICP': null},
                    member_threshold: 3,
                    wallet_ids: [[1n]]
                },

            },
            state: {'Archived': null},
            vault: 66n
        }
        console.log(policy)
        let result = await dfx.vault.actor.update_policy(updatePolicyRequest) as Policy
        policy.state = {'Archived': null};
        policy.policy_type = {
            'threshold_policy': {
                amount_threshold: 2n,
                currency: {'ICP': null},
                member_threshold: 3,
                wallet_ids: [[1n]]
            }
        };
        console.log(result)
        verifyPolicy(result, policy);
        expect(policy.modified_date !== result.modified_date).true
    })

    it("register policy negative ", async function () {
        try {
            await dfx.vault.actor_member.get_policies(2n)
        } catch (e: any) {
            expect(e.message.includes("Unauthorised")).true
        }
        try {
            await dfx.vault.actor_member.get_policies(3n)
        } catch (e: any) {
            expect(e.message.includes("Nonexistent key error")).true
        }
        try {
            await dfx.vault.actor_member.register_policy({
                policy_type: {
                    'threshold_policy': {
                        amount_threshold: 10n,
                        currency: {'ICP': null},
                        member_threshold: 2,
                        wallet_ids: []
                    }
                }, vault_id: 3n
            })
        } catch (e: any) {
            expect(e.message.includes("Nonexistent key")).true
        }
        try {
            await dfx.vault.actor_member.register_policy({
                policy_type: {
                    'threshold_policy': {
                        amount_threshold: 10n,
                        currency: {'ICP': null},
                        member_threshold: 2,
                        wallet_ids: []
                    }
                }, vault_id: 2n
            })
        } catch (e: any) {
            expect(e.message.includes("Unauthorised")).true
        }
        try {
            await dfx.vault.actor_member.register_policy({
                policy_type: {
                    'threshold_policy': {
                        amount_threshold: 10n,
                        currency: {'ICP': null},
                        member_threshold: 2,
                        wallet_ids: []
                    }
                }, vault_id: 1n
            })
        } catch (e: any) {
            expect(e.message.includes("Not enough permissions")).eq(true)
        }
    });
});


function verifyPolicy(actual: Policy, expected: Policy) {
    expect(actual.id).eq(expected.id)
    expect(actual.vault).eq(expected.vault)
    expect(Object.keys(actual.state)[0]).eq(Object.keys(expected.state)[0])
    expect(actual.policy_type.threshold_policy.member_threshold)
        .eq(expected.policy_type.threshold_policy.member_threshold)
    expect(actual.policy_type.threshold_policy.amount_threshold)
        .eq(expected.policy_type.threshold_policy.amount_threshold)
    expect(actual.policy_type.threshold_policy.wallet_ids.length)
        .eq(expected.policy_type.threshold_policy.wallet_ids.length)
    expect(Object.keys(actual.policy_type.threshold_policy.currency)[0])
        .eq(Object.keys(expected.policy_type.threshold_policy.currency)[0])
    if (actual.policy_type.threshold_policy.wallet_ids.length > 0) {
        for (const wallet of expected.policy_type.threshold_policy.wallet_ids as bigint[]) {
            expect((actual.policy_type.threshold_policy.wallet_ids as bigint[]).includes(wallet))
        }
    }
}