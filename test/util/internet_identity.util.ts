import { ActorMethod } from "@dfinity/agent";
import { Ed25519KeyIdentity } from "@dfinity/identity";
import { Challenge, ChallengeResult, DeviceData, UserNumber } from "../idl/internet_identity_test";

export const register = async (actor: Record<string, ActorMethod>, identity: Ed25519KeyIdentity): Promise<bigint> => {
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
    var registerResponse = (await actor.register(deviceData, challenageResult)) as { 'registered' : { 'user_number' : UserNumber } };;
    
    return registerResponse.registered.user_number;
};