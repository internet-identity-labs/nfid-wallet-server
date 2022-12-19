import "mocha";
import {deploy} from "../util/deployment.util";
import {Dfx} from "../type/dfx";
import {App} from "../constanst/app.enum";
import {
    Policy,
    PolicyRegisterRequest,
    ThresholdPolicy,
    Transaction,
    TransactionApproveRequest,
    TransactionRegisterRequest,
    TransactionState,
    Vault,
    VaultMemberRequest,
    VaultRegisterRequest,
    Wallet
} from "../idl/vault";
import {expect} from "chai";
import {fromHexString, principalToAddress, principalToAddressBytes} from "ictool"
import {DFX} from "../constanst/dfx.const";
import {Principal} from "@dfinity/principal";
import {call} from "../util/call.util";


describe("Transaction", () => {
    var dfx: Dfx;

    before(async () => {
        dfx = await deploy({apps: [App.Vault]});
    });

    after(() => {
        DFX.STOP();
    });


    it("Transaction register", async function () {
        let request: VaultRegisterRequest = {
            description: ["test"],
            name: "vault1"
        };

        await dfx.vault.actor.register_vault(request)
        let address = principalToAddress(dfx.user.identity.getPrincipal() as any, Array(32).fill(1));

        let memberAddress = principalToAddress(dfx.vault.member.getPrincipal() as any, Array(32).fill(1));

        let vaultMember: VaultMemberRequest = {
            address: memberAddress,
            name: ["MoyaLaskovayaSuchechka"],
            role: {'Member': null},
            vault_id: 1n,
            state: {'Active': null},
        }
        await dfx.vault.actor.store_member(vaultMember) as Vault;
        let wallet1 = await dfx.vault.actor.register_wallet({name: ["Wallet1"], vault_id: 1n}) as Wallet
        let walBytes = principalToAddressBytes(Principal.fromText(dfx.vault.id) as any, fromHexString(wallet1.uid))
        call(`dfx canister call ledger transfer "(record { to=vec { ${walBytes.toString().replaceAll(',', ';')} };
          amount=record { e8s=200_000_000 }; fee=record { e8s=10_000 }; memo=0:nat64; } )"`);

        let wallet2 = await dfx.vault.actor.register_wallet({name: ["Wallet2"], vault_id: 1n}) as Wallet
        let tp: ThresholdPolicy = {
            amount_threshold: 1n,
            currency: {'ICP': null},
            member_threshold: [2],
            wallets: []
        }
        let request3: PolicyRegisterRequest = {policy_type: {'threshold_policy': tp}, vault_id: 1n};
        let policy = await dfx.vault.actor.register_policy(request3) as Policy

        let tokens = 100000n
        let to = principalToAddress(Principal.fromText(dfx.vault.id) as any, fromHexString(wallet2.uid))

        let registerRequest: TransactionRegisterRequest = {address: to, amount: tokens, wallet_id: wallet1.uid}

        let actualTransaction = await dfx.vault.actor.register_transaction(registerRequest) as Transaction

        expect(actualTransaction.id).eq(1n)
        expect(actualTransaction.to).eq(to);
        expect(actualTransaction.amount_threshold).eq(1n);
        expect(actualTransaction.state.hasOwnProperty('Pending')).eq(true);
        expect(actualTransaction.approves.length).eq(1);
        expect(actualTransaction.approves[0].signer).eq(address);
        expect(actualTransaction.approves[0].status.hasOwnProperty("Approved")).eq(true);
        expect(actualTransaction.approves[0].created_date > 0).eq(true);
        expect(actualTransaction.created_date > 0).eq(true);
        expect(actualTransaction.modified_date > 0).eq(true);
        expect(actualTransaction.amount).eq(tokens);
        expect(actualTransaction.block_index.length).eq(0);
        expect(actualTransaction.currency.ICP).eq(null);
        expect(actualTransaction.policy_id).eq(policy.id);
        expect(actualTransaction.from).eq(wallet1.uid);

        let state = {'Approved': null} as TransactionState

        let approve: TransactionApproveRequest = {state: state, transaction_id: actualTransaction.id}

        let completed = await dfx.vault.actor_member.approve_transaction(approve) as Transaction
        expect(completed.id).eq(1n)
        expect(completed.to).eq(to);
        expect(completed.amount_threshold).eq(1n);
        expect(completed.state.hasOwnProperty('Approved')).eq(true);
        expect(completed.approves.length).eq(2);
        expect(completed.approves.find(l => l.signer === address).signer).eq(address);
        expect(completed.approves[0].status.hasOwnProperty("Approved")).eq(true);
        expect(completed.approves.find(l => l.signer === memberAddress).signer).eq(memberAddress); //TODO
        expect(completed.approves[1].status.hasOwnProperty("Approved")).eq(true);
        expect(completed.approves[0].created_date > 0).eq(true);
        expect(completed.approves[1].created_date > 0).eq(true);
        expect(completed.created_date > 0).eq(true);
        expect(completed.amount).eq(tokens);
        expect(completed.block_index[0]).eq(2n);
        expect(completed.currency.ICP).eq(null);
        expect(completed.policy_id).eq(policy.id);
        expect(completed.from).eq(wallet1.uid);
        expect(completed.memo.length).eq(0);
        expect(completed.modified_date > actualTransaction.modified_date).eq(true);
        expect(completed.created_date === actualTransaction.created_date).eq(true);

        let transactions = await dfx.vault.actor.get_transactions() as Array<Transaction>

        expect(transactions.length).eq(1)
        let transactionsMember = await dfx.vault.actor_member.get_transactions() as Array<Transaction>
        expect(transactionsMember.length).eq(1)
        expect(transactionsMember[0].id).eq(transactions[0].id)
    });

});