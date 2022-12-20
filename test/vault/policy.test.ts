import "mocha";
import {deploy} from "../util/deployment.util";
import {Dfx} from "../type/dfx";
import {App} from "../constanst/app.enum";
import {Policy, PolicyRegisterRequest} from "../idl/vault";
import {expect} from "chai";
import {principalToAddress} from "ictool"
import {DFX} from "../constanst/dfx.const";


let memberAddress: string;

describe("Policy", () => {
    var dfx: Dfx;

    before(async () => {
        dfx = await deploy({apps: [App.Vault]});
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
    it("verify default policy", async function () {
        let policies = await dfx.vault.actor.get_policies(1n) as [Policy]

        verifyPolicy(policies[0], {
            state: {'Active': null},
            vault: 1n,
            created_date: 0n,
            id: 1n,
            modified_date: 0n,
            policy_type: {
                'threshold_policy': {
                    amount_threshold: 0n,
                    currency: {'ICP': null},
                    member_threshold: [],
                    wallets: []
                },

            }
        })
    })

    it("register policy", async function () {

        let result1 = await dfx.vault.actor.register_policy({
            policy_type: {
                'threshold_policy': {
                    amount_threshold: 10n,
                    currency: {'ICP': null},
                    member_threshold: [2],
                    wallets: []
                }
            },
            state: {'Active': null},
            vault_id: 1n
        } as PolicyRegisterRequest) as Policy
        verifyPolicy(result1, {
            state: {'Active': null},
            vault: 1n,
            created_date: 0n,
            id: 3n,
            modified_date: 0n,
            policy_type: {
                'threshold_policy': {
                    amount_threshold: 10n,
                    currency: {'ICP': null},
                    member_threshold: [2],
                    wallets: []
                },

            }
        })
        expect(result1.created_date).eq(result1.modified_date);
        let result2 = await dfx.vault.actor.register_policy({
            policy_type: {
                'threshold_policy': {
                    amount_threshold: 2n,
                    currency: {'ICP': null},
                    member_threshold: [3],
                    wallets: [["some_uid"]]
                }
            },
            vault_id: 1n
        }) as Policy
        verifyPolicy(result2, {
            state: {'Active': null},
            vault: 1n,
            created_date: 0n,
            id: 4n,
            modified_date: 0n,
            policy_type: {
                'threshold_policy': {
                    amount_threshold: 2n,
                    currency: {'ICP': null},
                    member_threshold: [3],
                    wallets: [["test_uid"]]
                },
            }
        });
        let policies = await dfx.vault.actor.get_policies(1n) as [Policy]
        expect(policies.length).eq(3)
        let policy1 = policies.find(l => l.id === 3n)
        let policy2 = policies.find(l => l.id === 4n)
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
                    member_threshold: [3],
                    wallets: [["test_uid"]]
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
                member_threshold: [3],
                wallets: [["test_uid"]]
            }
        };
        console.log(result)
        verifyPolicy(result, policy);
        expect(policy.modified_date !== result.modified_date).true

        try {
            await dfx.vault.actor_member.update_policy(updatePolicyRequest)
        } catch (e: any) {
            expect(e.message.includes("Not enough permissions")).eq(true)
        }
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
                        member_threshold: [2],
                        wallets: []
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
                        member_threshold: [2],
                        wallets: []
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
                        member_threshold: [2],
                        wallets: []
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
    expect(actual.policy_type.threshold_policy.member_threshold.length)
        .eq(expected.policy_type.threshold_policy.member_threshold.length)
    expect(actual.policy_type.threshold_policy.amount_threshold)
        .eq(expected.policy_type.threshold_policy.amount_threshold)
    expect(actual.policy_type.threshold_policy.wallets.length)
        .eq(expected.policy_type.threshold_policy.wallets.length)
    expect(Object.keys(actual.policy_type.threshold_policy.currency)[0])
        .eq(Object.keys(expected.policy_type.threshold_policy.currency)[0])
    if (actual.policy_type.threshold_policy.wallets.length > 0
        && actual.policy_type.threshold_policy.wallets[0].length > 0) {
        for (const wallet of expected.policy_type.threshold_policy.wallets as string[]) {
            expect((actual.policy_type.threshold_policy.wallets as string[]).includes(wallet))
        }
    }
    if (actual.policy_type.threshold_policy.member_threshold.length > 0) {
        expect(actual.policy_type.threshold_policy.member_threshold[0])
            .eq(expected.policy_type.threshold_policy.member_threshold[0])
    }
}