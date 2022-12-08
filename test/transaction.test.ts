import "mocha";
import {deploy} from "./util/deployment.util";
import {Dfx} from "./type/dfx";
import {App} from "./constanst/app.enum";
import {
    Policy,
    PolicyRegisterRequest,
    State,
    ThresholdPolicy,
    Tokens,
    Transaction,
    Vault,
    VaultMemberRequest,
    VaultRegisterRequest,
    Wallet
} from "./idl/vault";
import {expect} from "chai";
import {principalToAddress} from "ictool"
import {DFX} from "./constanst/dfx.const";


describe("Transaction", () => {
    var dfx: Dfx;

    before(async () => {
        dfx = await deploy(App.Vault);
    });

    after(() => {
        DFX.STOP();
    });


    it("Transaction  register", async function () {
        let request = {
            name: "vault1"
        } as VaultRegisterRequest;

        let vault = await dfx.vault.actor.register_vault(request) as Vault
        let address = principalToAddress(dfx.user.identity.getPrincipal() as any, Array(32).fill(1));
        request = {
            name: "vault2"
        } as VaultRegisterRequest;
        let vault2 = await dfx.vault.actor.register_vault(request) as Vault

        let memberAddress = principalToAddress(dfx.vault.member.getPrincipal() as any, Array(32).fill(1));

        let vaultMember: VaultMemberRequest = {
            address: memberAddress,
            name: ["MoyaLaskovayaSuchechka"],
            role: {'Member': null},
            vault_id: 1n
        }
        await dfx.vault.actor.add_vault_member(vaultMember) as Vault;
        let wallet1 = await dfx.vault.actor.register_wallet({name: ["Wallet1"], vault_id: 1n}) as Wallet
        console.log()
        let wallet2 = await dfx.vault.actor.register_wallet({name: ["Wallet2"], vault_id: 1n}) as Wallet
        let tp: ThresholdPolicy = {
            amount_threshold: 1n,
            currency: {'ICP': null},
            member_threshold: 2,
            wallet_ids: []
        }
        let request3: PolicyRegisterRequest = {policy_type: {'threshold_policy': tp}, vault_id: 1n};
        let policy = await dfx.vault.actor.register_policy(request3) as Policy

        let tokens: Tokens = {
            e8s: 100000n

        }
        let wallet2Address = await dfx.vault.actor.sub(2);
        console.log(await dfx.vault.actor.sub(1));
        let actualTransaction = await dfx.vault.actor.register_transaction(tokens, wallet2Address, 1) as Transaction

        expect(actualTransaction.id).eq(1n)
        expect(actualTransaction.to).eq(wallet2Address);
        expect(actualTransaction.amount_threshold).eq(1n);
        expect(actualTransaction.state.hasOwnProperty('PENDING')).eq(true);
        expect(actualTransaction.approves.length).eq(1);
        expect(actualTransaction.approves[0].signer).eq(address);
        expect(actualTransaction.approves[0].status.hasOwnProperty("APPROVED")).eq(true);
        expect(actualTransaction.approves[0].created_date > 0).eq(true);
        expect(actualTransaction.created_date > 0).eq(true);
        expect(actualTransaction.modified_date > 0).eq(true);
        expect(actualTransaction.amount.e8s).eq(tokens.e8s);
        expect(actualTransaction.block_index.length).eq(0);
        expect(actualTransaction.currency.ICP).eq(null);
        expect(actualTransaction.policy_id).eq(policy.id);
        expect(actualTransaction.wallet_id).eq(wallet1.id);

        let state = {'APPROVED': null} as State
        let completed = await dfx.vault.actor_member.approve_transaction(actualTransaction.id, state) as Transaction
        expect(completed.id).eq(1n)
        expect(completed.to).eq(wallet2Address);
        expect(completed.amount_threshold).eq(1n);
        expect(completed.state.hasOwnProperty('APPROVED')).eq(true);
        expect(completed.approves.length).eq(2);
        expect(completed.approves[0].signer).eq(address);
        expect(completed.approves[0].status.hasOwnProperty("APPROVED")).eq(true);
        expect(completed.approves[1].signer).eq(memberAddress);
        expect(completed.approves[1].status.hasOwnProperty("APPROVED")).eq(true);
        expect(completed.approves[0].created_date > 0).eq(true);
        expect(completed.approves[1].created_date > 0).eq(true);
        expect(completed.created_date > 0).eq(true);
        expect(completed.amount.e8s).eq(tokens.e8s);
        expect(completed.block_index[0]).eq(2n);
        expect(completed.currency.ICP).eq(null);
        expect(completed.policy_id).eq(policy.id);
        expect(completed.wallet_id).eq(wallet1.id);
        expect(completed.modified_date > actualTransaction.modified_date).eq(true);
        expect(completed.created_date === actualTransaction.created_date).eq(true);

        let transactions = await dfx.vault.actor.get_transactions() as Array<Transaction>

        expect(transactions.length).eq(1)
        let transactionsMember = await dfx.vault.actor_member.get_transactions() as Array<Transaction>
        expect(transactionsMember.length).eq(1)
        expect(transactionsMember[0].id).eq(transactions[0].id)
    });



});


// A `hasOwnProperty` that produces evidence for the typechecker
export function hasOwnProperty<X extends Record<string, unknown>,
    Y extends PropertyKey,
    >(obj: X, prop: Y): obj is X & Record<Y, unknown> {
    return Object.prototype.hasOwnProperty.call(obj, prop)
}