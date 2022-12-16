import "mocha";
import {deploy} from "../util/deployment.util";
import {Dfx} from "../type/dfx";
import {App} from "../constanst/app.enum";
import {Wallet} from "../idl/vault";
import {expect} from "chai";
import {principalToAddress} from "ictool"
import {DFX} from "../constanst/dfx.const";

let memberAddress: string;
describe("Wallet", () => {
    let dfx: Dfx;

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


    it("wallet register", async function () {

        let result1 = await dfx.vault.actor.register_wallet({name: ["Wallet1"], vault_id: 1n}) as Wallet
        verifyWallet(result1, {
            created_date: 0n,
            id: 1n,
            modified_date: 0n,
            name: ["Wallet1"],
            state: {'Active': null},
            vaults: [1n]

        })
        let result2 = await dfx.vault.actor.register_wallet({name: ["Wallet2"], vault_id: 1n}) as Wallet
        verifyWallet(result2, {
            created_date: 0n,
            id: 2n,
            modified_date: 0n,
            name: ["Wallet2"],
            state: {'Active': null},
            vaults: [1n]

        })
        let wallets = await dfx.vault.actor.get_wallets(1n) as [Wallet]
        expect(wallets.length).eq(2)
        let wallet1 = wallets.find(l => l.id === 1n)
        let wallet2 = wallets.find(l => l.id === 2n)
        verifyWallet(wallet1, {
            created_date: 0n,
            id: 1n,
            modified_date: 0n,
            name: ["Wallet1"],
            state: {'Active': null},
            vaults: [1n]

        })
        verifyWallet(wallet2, {
            created_date: 0n,
            id: 2n,
            modified_date: 0n,
            name: ["Wallet2"],
            state: {'Active': null},
            vaults: [1n]
        })
    });
    it("update wallet", async function () {
        let wallets = await dfx.vault.actor.get_wallets(1n) as [Wallet]
        let wallet1 = wallets.find(l => l.id === 1n)
        let updated = await dfx.vault.actor.update_wallet({
            created_date: 321n,
            id: 1n,
            modified_date: 123n,
            name: ["Wallet1_Udated"],
            state: {'Archived': null},
            vaults: [2n]

        }) as Wallet
        wallet1.name = ["Wallet1_Udated"]
        wallet1.state = {'Archived': null}
        verifyWallet(wallet1, updated)
        expect(wallet1.modified_date !== updated.modified_date).true
    })

    it("register wallet negative ", async function () {
        try {
            await dfx.vault.actor_member.get_wallets(2n)
        } catch (e: any) {
            expect(e.message.includes("Unauthorised")).eq(true)
        }
        try {
            await dfx.vault.actor_member.get_wallets(3n)
        } catch (e: any) {
            expect(e.message.includes("Nonexistent key")).eq(true)
        }
        try {
            await dfx.vault.actor_member.register_wallet({name: ["Wallet1"], vault_id: 3n})
        } catch (e: any) {
            expect(e.message.includes("Nonexistent key")).eq(true)
        }
        try {
            await dfx.vault.actor_member.register_wallet({name: ["Wallet1"], vault_id: 2n})
        } catch (e: any) {
            expect(e.message.includes("Unauthorised")).eq(true)
        }
        try {
            await dfx.vault.actor_member.register_wallet({name: ["Wallet1"], vault_id: 1n})
        } catch (e: any) {
            expect(e.message.includes("Not enough permissions")).eq(true)
        }
    });
});


function verifyWallet(actual: Wallet, expected: Wallet) {
    expect(actual.vaults.length).eq(expected.vaults.length)
    if (actual.vaults.length > 0) {
        expect(actual.vaults[0]).eq(expected.vaults[0])
    }
    expect(actual.name.length).eq(expected.name.length)
    if (actual.name.length > 0) {
        expect(actual.name[0]).eq(expected.name[0])
    }
    expect(actual.id).eq(expected.id)
    expect(Object.keys(actual.state)[0]).eq(Object.keys(expected.state)[0])
}