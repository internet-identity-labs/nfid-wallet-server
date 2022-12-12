import "mocha";
import {deploy} from "./util/deployment.util";
import {Dfx} from "./type/dfx";
import {App} from "./constanst/app.enum";
import {
    Policy,
    PolicyRegisterRequest, ThresholdPolicy,
    Vault,
    VaultMember,
    VaultMemberRequest,
    VaultRegisterRequest,
    Wallet,
    WalletRegisterRequest
} from "./idl/vault";
import {expect} from "chai";
import {principalToAddress} from "ictool"
import {DFX} from "./constanst/dfx.const";


describe("Vault", () => {
    var dfx: Dfx;

    before(async () => {
        dfx = await deploy(App.Vault);
    });

    after(() => {
        DFX.STOP();
    });


    it("vault get empty", async function () {
        let vaults = await dfx.vault.actor.get_vaults() as [Vault]
        expect(vaults.length).eq(0)
    });


    it("vault  register", async function () {
        let request: VaultRegisterRequest = {
            description: ["test"],
            name: "vault1"
        };

        let vault = await dfx.vault.actor.register_vault(request) as Vault
        let address = principalToAddress(dfx.user.identity.getPrincipal() as any, Array(32).fill(1));
        expect(vault.name).eq("vault1")
        expect(vault.members.length).eq(1)
        expect(vault.members[0].user_uuid).eq(address)
        expect(vault.members[0].role.hasOwnProperty('Admin')).eq(true)
        request = {
            description: ["test2"],
            name: "vault2"
        };
        vault = await dfx.vault.actor.register_vault(request) as Vault
        expect(vault.name).eq("vault2")
        expect(vault.members.length).eq(1)
        expect(vault.members[0].user_uuid).eq(address)
        expect(vault.members[0].role.hasOwnProperty('Admin')).eq(true)
    });

    it("vault  get by ids", async function () {
        let vaults = await dfx.vault.actor.get_vaults() as [Vault]
        expect(vaults.length).eq(2)
        expect(vaults[0].id).eq(2n)
        // @ts-ignore
        expect(vaults[1].id).eq(1n)
    });

    it("add member/ get members", async function () {
        let memberAddress = principalToAddress(dfx.vault.member.getPrincipal() as any, Array(32).fill(1));

        let vaultMember: VaultMemberRequest = {
            address: memberAddress,
            name: ["MoyaLaskovayaSuchechka"],
            role: {'Member': null},
            vault_id: 1n
        }
        let vault = await dfx.vault.actor.add_vault_member(vaultMember) as Vault;
        let actual = vault.members.find(l => l.user_uuid === memberAddress);
        expect(actual.name[0]).eq("MoyaLaskovayaSuchechka")
        expect(actual.role.hasOwnProperty('Member')).eq(true)

        let vaultForMember = (await dfx.vault.actor_member.get_vaults())[0] as Vault;
        expect(vaultForMember.name).eq(vault.name)
        expect(vaultForMember.id).eq(vault.id)
        expect(vaultForMember.wallets.length).eq(vault.wallets.length)
        expect(vaultForMember.policies.length).eq(vault.policies.length)
        expect(vaultForMember.members.length).eq(2)
        expect(vaultForMember.members[0].user_uuid).eq(vault.members[0].user_uuid)
        expect(vaultForMember.members[1].user_uuid).eq(vault.members[1].user_uuid)

        let membersForAdmin = (await dfx.vault.actor.get_vaults() as [Vault])
            .filter(l => l.id === 1n)[0].members
        let membersForMember = (await dfx.vault.actor_member.get_vaults() as [Vault])
            .filter(l => l.id === 1n)[0].members
        expect(membersForAdmin.length).eq(2)
        expect(membersForMember.length).eq(2)
        expect(membersForMember[0].user_uuid).eq(membersForAdmin[0].user_uuid)
        expect(JSON.stringify(membersForMember[0].role)).eq(JSON.stringify(membersForAdmin[0].role))
        // @ts-ignore
        expect(membersForMember[1].user_uuid).eq(membersForAdmin[1].user_uuid)
        // @ts-ignore
        expect(JSON.stringify(membersForMember[1].role)).eq(JSON.stringify(membersForAdmin[1].role))
    });
    it("negative scenarios for add members", async function () {
        let memberAddress = principalToAddress(dfx.vault.member.getPrincipal() as any, Array(32).fill(1));
        let vaultMember: VaultMemberRequest = {
            address: memberAddress,
            name: ["Moya Laskovaya Suchechka"],
            role: {'Member': null},
            vault_id: 2n
        }
        try {
            await dfx.vault.actor_member.add_vault_member(vaultMember)
        } catch (e: any) {
            expect(e.message.includes("Unauthorised")).eq(true)
        }
        vaultMember = {
            address: memberAddress,
            name: ["Moya Laskovaya Suchechka"],
            role: {'Member': null},
            vault_id: 1n
        }
        try {
            await dfx.vault.actor_member.add_vault_member(vaultMember)
        } catch (e: any) {
            expect(e.message.includes("Not enough permissions")).eq(true)
        }
    });
    it("register wallet", async function () {
        let request: WalletRegisterRequest = {name: ["Wallet1"], vault_id: 1n};
        let result = await dfx.vault.actor.register_wallet(request) as Wallet
        expect(result.name[0]).eq("Wallet1")
        expect(result.vaults[0]).eq(1n)
        expect(result.id).eq(1n)
        request = {name: ["Wallet2"], vault_id: 1n};
        result = await dfx.vault.actor.register_wallet(request) as Wallet
        expect(result.name[0]).eq("Wallet2")
        expect(result.vaults[0]).eq(1n)
        expect(result.id).eq(2n)
        let wallets = await dfx.vault.actor.get_wallets(1n) as [Wallet]
        expect(wallets.length).eq(2)
        expect(wallets[0].id).eq(1n)
        // @ts-ignore
        expect(wallets[1].id).eq(2n)
    });

    it("register wallet negative ", async function () {
        try {
            await dfx.vault.actor_member.get_wallets(2n)
        } catch (e: any) {
            expect(e.message.includes("Unauthorised")).eq(true)
        }
        try {
            await dfx.vault.actor_member.get_wallets(3n)
        } catch (e: any) {
            console.log(e)
            expect(e.message.includes("Nonexistent key")).eq(true)
        }
        let request: WalletRegisterRequest = {name: ["Wallet1"], vault_id: 3n};
        try {
            await dfx.vault.actor_member.register_wallet(request) as Wallet
        } catch (e: any) {
            expect(e.message.includes("Nonexistent key")).eq(true)
        }

        request = {name: ["Wallet1"], vault_id: 2n};
        try {
            await dfx.vault.actor_member.register_wallet(request) as Wallet
        } catch (e: any) {
            expect(e.message.includes("Unauthorised")).eq(true)
        }

        request = {name: ["Wallet1"], vault_id: 1n};
        try {
            await dfx.vault.actor_member.register_wallet(request) as Wallet
        } catch (e: any) {
            expect(e.message.includes("Not enough permissions")).eq(true)
        }
    });


    it("register policy", async function () {
        let tp: ThresholdPolicy = {
            amount_threshold: 10n,
            currency: {'ICP': null},
            member_threshold: 2,
            wallet_ids: []
        }
        let request: PolicyRegisterRequest = {policy_type: {'threshold_policy': tp}, vault_id: 1n};

        let result = await dfx.vault.actor.register_policy(request) as Policy
        expect(result.id).eq(1n)
        expect(result.policy_type.threshold_policy.amount_threshold).eq(10n)
        expect(result.policy_type.threshold_policy.member_threshold).eq(2)
        expect(result.policy_type.threshold_policy.wallet_ids.length).eq(0)
        expect(hasOwnProperty(result.policy_type.threshold_policy.currency, 'ICP')).eq(true)
        tp = {
            amount_threshold: 2n,
            currency: {'ICP': null},
            member_threshold: 3,
            wallet_ids: []
        }
        request = {policy_type: {'threshold_policy': tp}, vault_id: 1n};

        result = await dfx.vault.actor.register_policy(request) as Policy
        expect(result.id).eq(2n)
        let policies = await dfx.vault.actor.get_policies(1n) as [Policy]
        expect(policies.length).eq(2)
        expect(policies[0].id).eq(1n)
        // @ts-ignore
        expect(policies[1].id).eq(2n)
    });


    it("register policy negative ", async function () {
        try {
            await dfx.vault.actor_member.get_policies(2n)
        } catch (e: any) {
            expect(e.message.includes("Unauthorised")).eq(true)
        }
        try {
            await dfx.vault.actor_member.get_policies(3n)
        } catch (e: any) {
            expect(e.message.includes("Nonexistent key error")).eq(true)
        }

        let tp: ThresholdPolicy = {
            amount_threshold: 10n,
            currency: {'ICP': null},
            member_threshold: 2,
            wallet_ids: []
        }
        let request: PolicyRegisterRequest = {policy_type: {'threshold_policy': tp}, vault_id: 3n};

        try {
            await dfx.vault.actor_member.register_policy(request)
        } catch (e: any) {
            expect(e.message.includes("Nonexistent key")).eq(true)
        }

        request = {policy_type: {'threshold_policy': tp}, vault_id: 2n};
        try {
            await dfx.vault.actor_member.register_policy(request)
        } catch (e: any) {
            expect(e.message.includes("Unauthorised")).eq(true)
        }

        request = {policy_type: {'threshold_policy': tp}, vault_id: 1n};
        try {
            await dfx.vault.actor_member.register_policy(request)
        } catch (e: any) {
            expect(e.message.includes("Not enough permissions")).eq(true)
        }
    });
});


// A `hasOwnProperty` that produces evidence for the typechecker
export function hasOwnProperty<X extends Record<string, unknown>,
    Y extends PropertyKey,
    >(obj: X, prop: Y): obj is X & Record<Y, unknown> {
    return Object.prototype.hasOwnProperty.call(obj, prop)
}