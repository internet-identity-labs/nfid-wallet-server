import "mocha";
import {deploy} from "./util/deployment.util";
import {Dfx} from "./type/dfx";
import {App} from "./constanst/app.enum";
import {Vault, VaultMember, VaultMemberRequest, VaultRegisterRequest} from "./idl/vault";
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
        let request = {
            name: "vault1"
        } as VaultRegisterRequest;

        let vault = await dfx.vault.actor.register_vault(request) as Vault
        let address = principalToAddress(dfx.user.identity.getPrincipal() as any, Array(32).fill(1));
        expect(vault.name).eq("vault1")
        expect(vault.members.length).eq(1)
        expect(vault.members[0].user_uuid).eq(address)
        expect(vault.members[0].role.hasOwnProperty('Admin')).eq(true)
        request = {
            name: "vault2"
        } as VaultRegisterRequest;
        vault = await dfx.vault.actor.register_vault(request) as Vault
        expect(vault.name).eq("vault2")
        expect(vault.members.length).eq(1)
        expect(vault.members[0].user_uuid).eq(address)
        expect(vault.members[0].role.hasOwnProperty('Admin')).eq(true)
    });

    it("vault  get by ids", async function () {
        let vaults = await dfx.vault.actor.get_vaults() as [Vault]
        expect(vaults.length).eq(2)
        expect(vaults[0].id).eq(1n)
        // @ts-ignore
        expect(vaults[1].id).eq(2n)
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

        let membersForAdmin = await dfx.vault.actor.get_vault_members(1n) as [VaultMember]
        let membersForMember = await dfx.vault.actor_member.get_vault_members(1n) as [VaultMember]

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
        try {
            await dfx.vault.actor_member.get_vault_members(2n)
        } catch (e: any) {
            expect(e.message.includes("Not participant")).eq(true)
        }
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
});