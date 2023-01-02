"use strict";

import "mocha";
import {Secp256k1KeyIdentity} from "@dfinity/identity";
import {JsonnableEd25519KeyIdentity} from "@dfinity/identity/lib/cjs/identity/ed25519";
import {BigNumber, Bytes, Signer, UnsignedTransaction, Wallet} from "ethers";
import {Actor, HttpAgent} from "@dfinity/agent";
import {idlFactory as ecdsaIdl} from "./idl/ecdsa_idl";
import {Provider, TransactionRequest} from "@ethersproject/abstract-provider";
import "./elliptic-types";
import * as elliptic from "elliptic";
import {KeyPair, Signature} from "elliptic";
import {toHexString} from "ictool";
import {arrayify, hexZeroPad, joinSignature, splitSignature} from "@ethersproject/bytes";
import {BN} from "bn.js";
import {hashMessage, keccak256, resolveProperties} from "ethers/lib/utils";
import {ActorMethod, ActorSubclass} from "@dfinity/agent/lib/esm/actor";
import {serialize} from "@ethersproject/transactions";

var ethers = require('ethers');

var EC = elliptic.ec;
var ec = new EC('secp256k1');

describe("1111", () => {
    const idd: JsonnableEd25519KeyIdentity = [
        "0402f7e13e782ad8bb2c4da69d00c14af52d4bf0f1cc20ddb52f117d7fff2e3678c950145102d87915c5688a218cdc4348407cd7b1fdb8256dade044309a2552cd",
        "fa1e290e2524ec98e24e49e95c0e30b43e9c96504715cea4d802269f80f638e6"
    ];


    describe("111", async () => {

            var customHttpProvider = new ethers.providers.JsonRpcProvider("http://127.0.0.1:8545");
            let sk: string = idd[1]
            let wallet = new Wallet(sk, customHttpProvider)


            const addressFrom = ethers.utils.computeAddress(wallet.publicKey);
            let gasPrice = await customHttpProvider.getGasPrice()


            let nfidWallet = new NfidWallet(Secp256k1KeyIdentity.fromParsedJson(idd), customHttpProvider)
            await nfidWallet.init();
            let address = await nfidWallet.getAddress();

            const addressTo = address;

            let value = ethers.utils.parseEther("0.0000001")
            let gasLimit = BigNumber.from(100000)
            let tr_count = await customHttpProvider.getTransactionCount(address, "latest")
            const tx_from = {
                from: addressTo,
                to: addressFrom,
                value: value,
                nonce: tr_count,
                gasLimit: gasLimit,
                gasPrice: gasPrice,
            }
            let transactionFromNfid;
            try {
                transactionFromNfid = await nfidWallet.sendTransaction(tx_from);
            } catch (e) {
                console.log(e)
            }

            console.log(transactionFromNfid)


        }
    )

});


export class NfidWallet<T = Record<string, ActorMethod>> extends Signer {
    private readonly principal: Secp256k1KeyIdentity
    public actor: ActorSubclass<T>
    readonly provider: Provider;
    private address: string = undefined;

    constructor(identity: Secp256k1KeyIdentity, provider?: Provider) {
        super();
        this.principal = identity;
        this.provider = provider
    }

    async init() {
        const agent: HttpAgent = new HttpAgent({host: "http://127.0.0.1:8000", identity: this.principal});
        await agent.fetchRootKey();
        this.actor = await Actor.createActor(ecdsaIdl, {agent, canisterId: "rrkah-fqaaa-aaaaa-aaaaq-cai"});
    }

    async getAddress(): Promise<string> {
        if (typeof this.address !== "undefined") {
            return this.address
        }
        // @ts-ignore
        return this.actor.public_key()
            .then(l => {
                let canister_pk = l.Ok.public_key
                this.address = ethers.utils.computeAddress(canister_pk)
                return this.address
            })
    }

    async getPK(): Promise<KeyPair> {
        // @ts-ignore
        return this.actor.public_key()
            .then(l => {
                let canister_pk = l.Ok.public_key
                return ec.keyFromPublic(toHexString(canister_pk), 'hex')
            })
    }


    async signMessage(message: Bytes | string): Promise<string> {
        let keccak = hashMessage(message)
        const digestBytes = arrayify(keccak);
        // @ts-ignore
        return this.actor.sign([...digestBytes])
            .then(l => {
                console.log(l)
                let signature = l.Ok.signature
                let elliptic_signature = this.toEllipticSignature(signature)

                let ethersSignature = splitSignature({
                    recoveryParam: elliptic_signature.recoveryParam,
                    r: hexZeroPad("0x" + elliptic_signature.r.toString(16), 32),
                    s: hexZeroPad("0x" + elliptic_signature.s.toString(16), 32),
                })
                // @ts-ignore
                return joinSignature(ethersSignature);
            })
    }

    async signMessageRaw(message: Bytes | string): Promise<string> {
        let keccak = hashMessage(message)
        const digestBytes = arrayify(keccak);
        // @ts-ignore
        return this.actor.sign([...digestBytes])
            .then(l => {
                console.log(l)
                let signature = l.Ok.signature
                return this.toEllipticSignature(signature)
            })
    }

    async signTransaction(transaction: TransactionRequest): Promise<string> {
        return resolveProperties(transaction).then((tx) => {
            if (tx.from != null) {
                delete tx.from;
            }
            let keccak = keccak256(serialize(<UnsignedTransaction>tx));
            const digestBytes = arrayify(keccak);
            // @ts-ignore
            return this.actor.sign([...digestBytes])
                .then(response => {
                    let signature = response.Ok.signature
                    let elliptic_signature: Signature = this.toEllipticSignature(signature)
                    let ethersSignature = splitSignature({
                        recoveryParam: elliptic_signature.recoveryParam,
                        r: hexZeroPad("0x" + elliptic_signature.r.toString(16), 32),
                        s: hexZeroPad("0x" + elliptic_signature.s.toString(16), 32),
                    })
                    let address = ethers.utils.recoverAddress([...digestBytes], ethersSignature)
                    let isEqualAddress = address === this.address
                    if (!isEqualAddress) {
                        ethersSignature = splitSignature({
                            recoveryParam: 1 - (ethersSignature.recoveryParam % 2),
                            r: hexZeroPad("0x" + elliptic_signature.r.toString(16), 32),
                            s: hexZeroPad("0x" + elliptic_signature.s.toString(16), 32),
                        })
                    }
                    return serialize(<UnsignedTransaction>tx, ethersSignature)
                })
        });
    }

    toEllipticSignature(signature) {
        let bytes: Uint8Array = arrayify(signature)
        let v = 27 + (bytes[32] >> 7);
        // Allow a recid to be used as the v
        if (v < 27) {
            if (v === 0 || v === 1) {
                v += 27;
            }
        }
        // Compute recoveryParam from v
        let recoveryParam = 1 - (v % 2);
        let sig: Signature = {
            r: new BN(new Uint8Array(signature.slice(0, 32))),
            recoveryParam: recoveryParam,
            s: new BN(new Uint8Array(signature.slice(32, 64))),
        }
        return sig
    }


    connect(provider: Provider): Signer {
        return undefined;
    }


}

//

//
// var web3Provider = new Web3.providers.HttpProvider("http://127.0.0.1:8545");
// var web = new Web3(web3Provider);
// console.log(123)
// let aa = await web.eth.getBlockNumber()
// let bb = await web.eth.getAccounts()


// @ts-ignore
// let hex = toHexString(public_key)
// console.log(hex)

// const privKey =  Buffer.from(fromHexString(idd[1]))
// const publicKey =  Buffer.from(fromHexString(idd[0]))
//  let signature_secp = secp256k1.ecdsaSign(new Uint8Array(arr_hash), privKey).signature
//  let pub_key_secp = secp256k1.publicKeyCreate(privKey)
// let verified =  secp256k1.ecdsaVerify(signature_secp, new Uint8Array(arr_hash), publicKey)


// let ppsps = await getPkAndSignature(arr_hash)
//
// let cnister_pk = ppsps[0]
// let cnister_signature: Array<number> = ppsps[1]
//
//
// let elliptic_canister_public = ec.keyFromPublic(toHexString(cnister_pk), 'hex')
//
// let bytes: Uint8Array = arrayify(cnister_signature)
// let v = 27 + (bytes[32] >> 7);
// // Allow a recid to be used as the v
// if (v < 27) {
//     if (v === 0 || v === 1) {
//         v += 27;
//     }
// }
// // Compute recoveryParam from v
// let recoveryParam = 1 - (v % 2);
// let ssss: Signature = {
//     r: new BN(new Uint8Array(cnister_signature.slice(0, 32))),
//     recoveryParam: recoveryParam,
//     s: new BN(new Uint8Array(cnister_signature.slice(32, 64))),
// }
// let verify_canister_signature = ec.verify(new Uint8Array(arr_hash), ssss, elliptic_canister_public, 'hex')
// // var key = ec.keyFromPublic(pub, 'hex');
//
// console.log(verify_canister_signature)