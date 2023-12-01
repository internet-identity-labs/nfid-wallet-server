import { ActorMethod } from "@dfinity/agent";
import { Ed25519KeyIdentity } from "@dfinity/identity";
import { Challenge, ChallengeResult, DeviceData, UserNumber } from "../idl/internet_identity_test";
import * as Agent from "@dfinity/agent"
import {_SERVICE as InternetIdentityTest} from "../idl/internet_identity_test";

export const register = async (actor: Agent.ActorSubclass<InternetIdentityTest>, identity: Ed25519KeyIdentity): Promise<bigint> => {
    var challenge: Challenge = await actor.create_challenge() as Challenge;
    var challenageResult: ChallengeResult = {
        key: challenge.challenge_key,
        chars: 'a'
    };
    var deviceData: DeviceData = {
        alias: "Device",
        protection: {
            unprotected: null
        },
        pubkey: Array.from(new Uint8Array(identity.getPublicKey().toDer())),
        key_type: {
            platform: null
        },
        purpose: {
            authentication: null
        },
        credential_id: []
    };
    console.log(identity.getPrincipal().toText())
    var registerResponse = (await actor.register(deviceData, challenageResult)) as { 'registered' : { 'user_number' : UserNumber } };
    
    return registerResponse.registered.user_number;
};