"use strict";

import "mocha";
import {Secp256k1KeyIdentity} from "@dfinity/identity";
import {JsonnableEd25519KeyIdentity} from "@dfinity/identity/lib/cjs/identity/ed25519";
import {Bytes, Signer, Wallet} from "ethers";
import {Actor, HttpAgent} from "@dfinity/agent";
import {idlFactory as ecdsaIdl} from "./idl/ecdsa_idl";
import {Provider, TransactionRequest} from "@ethersproject/abstract-provider";
import "./elliptic-types";
import * as elliptic from "elliptic";
import {KeyPair, Signature} from "elliptic";
import {toHexString} from "ictool";
import {arrayify, hexZeroPad, joinSignature, splitSignature} from "@ethersproject/bytes";
import {BN} from "bn.js";
import {hashMessage} from "ethers/lib/utils";
import {ActorMethod, ActorSubclass} from "@dfinity/agent/lib/esm/actor";
import {assert} from "chai";

var Tx = require('ethereumjs-tx').Transaction;

var ethers = require('ethers');


var EC = elliptic.ec;
var ec = new EC('secp256k1');

describe("1111", () => {
    const idd: JsonnableEd25519KeyIdentity = [
        "0402f7e13e782ad8bb2c4da69d00c14af52d4bf0f1cc20ddb52f117d7fff2e3678c950145102d87915c5688a218cdc4348407cd7b1fdb8256dade044309a2552cd",
        "fa1e290e2524ec98e24e49e95c0e30b43e9c96504715cea4d802269f80f638e6"
    ];


    describe("111", async () => {

        var rawTx = {
            nonce: '0x00',
            gasPrice: '0x09184e72a000',
            gasLimit: '0x2710',
            to: '0x0000000000000000000000000000000000000000',
            value: '0x00',
            data: '0x7f7465737432000000000000000000000000000000000000000000000000000000600057'
        }
        var tx = new Tx(rawTx);
        let hash = tx.hash(false);
        let arr_hash = [...hash]


        let sk: string = idd[1]
        let wallet2 = new Wallet(sk)
        let eth_signature = await wallet2.signMessage(arr_hash)


        let nfidWallet = new NfidWallet(Secp256k1KeyIdentity.fromParsedJson(idd))
        await nfidWallet.init();

        let address = await nfidWallet.getAddress();

        console.log(address)

        let sian = await nfidWallet.signMessage("test_message")

        console.log(sian)

        let signRaw = await nfidWallet.signMessageRaw("test_message")

        let pk = await nfidWallet.getPK();

        console.log(pk)

        let m = hashMessage("test_message")
        const digestBytes = arrayify(m);
        let verify = ec.verify([...digestBytes], signRaw, pk, 'hex')
        assert(verify)
    });

});


export class NfidWallet<T = Record<string, ActorMethod>> extends Signer {
    private readonly principal: Secp256k1KeyIdentity
    public actor: ActorSubclass<T>

    constructor(identity: Secp256k1KeyIdentity) {
        super();
        this.principal = identity;
    }

    async init() {
        const agent: HttpAgent = new HttpAgent({host: "http://127.0.0.1:8000", identity: this.principal});
        await agent.fetchRootKey();
        this.actor = await Actor.createActor(ecdsaIdl, {agent, canisterId: "rrkah-fqaaa-aaaaa-aaaaq-cai"});
    }

    async getAddress(): Promise<string> {
        // @ts-ignore
        return this.actor.public_key()
            .then(l => {
                let canister_pk = l.Ok.public_key
                return ethers.utils.computeAddress(canister_pk)
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
        // return resolveProperties(transaction).then((tx) => {
        //     if (tx.from != null) {
        //         delete tx.from;
        //     }
        //     let keccak = keccak256(serialize(<UnsignedTransaction>tx));
        //     const digestBytes = arrayify(keccak);
        //     // @ts-ignore
        //     let response = await this.actor.sign(digestBytes)
        //     let signature = response.Ok.signature
        //     let elliptic_signature: Signature = this.toEllipticSignature(signature)
        //     let ethersSignature = splitSignature({
        //         recoveryParam: elliptic_signature.recoveryParam,
        //         r: hexZeroPad("0x" + elliptic_signature.r.toString(16), 32),
        //         s: hexZeroPad("0x" + elliptic_signature.s.toString(16), 32),
        //     })
        //     return serialize(<UnsignedTransaction>tx, ethersSignature);
        // });
        return undefined;
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