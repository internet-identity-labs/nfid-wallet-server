import "mocha";
import {deploy} from "./util/deployment.util";
import {Dfx} from "./type/dfx";
import {App} from "./constanst/app.enum";
import {DFX} from "./constanst/dfx.const";
import {Vault} from "./idl/vault";
import {expect} from "chai";
import { principalToAddress } from "ictool"


describe("Vault", () => {
    var dfx: Dfx;

    before(async () => {
        dfx = await deploy(App.Vault);
    });

    after(() => {
        // DFX.STOP();
    });

    it("vault  register", async function () {
        let vault = await dfx.vault.actor.register_vault("vault1") as Vault
        let address = principalToAddress(dfx.user.identity.getPrincipal() as any,  Array(32).fill(1));
        expect(vault.name).eq("vault1")
        expect(vault.members.length).eq(1)
        expect(vault.members[0].user_uuid).eq(address)
        expect(vault.members[0].role.hasOwnProperty('VaultOwner')).eq( true)
        vault = await dfx.vault.actor.register_vault("vault2") as Vault
        expect(vault.name).eq("vault2")
        expect(vault.members.length).eq(1)
        expect(vault.members[0].user_uuid).eq(address)
        expect(vault.members[0].role.hasOwnProperty('VaultOwner')).eq( true)
    });

    it("vault  get by ids", async function () {
        let vaults = await dfx.vault.actor.get_vaults() as [Vault]
        expect(vaults.length).eq(2)
        expect(vaults[0].id).eq(1n)
        // @ts-ignore
        expect(vaults[1].id).eq(2n)
    });

    it("create wallet", async function () {
        let vaults = await dfx.vault.actor.get_vaults() as [Vault]
        expect(vaults.length).eq(2)
        expect(vaults[0].id).eq(1n)
        // @ts-ignore
        expect(vaults[1].id).eq(2n)
    });

});