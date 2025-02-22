export const idlFactory = ({ IDL }) => {
    const KeyPair = IDL.Record({
        'public_key' : IDL.Text,
        'private_key_encrypted' : IDL.Text,
    });
    const KeyPairResponse = IDL.Record({
        'key_pair' : IDL.Opt(KeyPair),
        'princ' : IDL.Text,
    });
    const CertifiedKeyPairResponse = IDL.Record({
        'certificate' : IDL.Vec(IDL.Nat8),
        'witness' : IDL.Vec(IDL.Nat8),
        'response' : KeyPairResponse,
    });
    const SignatureReply = IDL.Record({ 'signature' : IDL.Vec(IDL.Nat8) });
    const Result = IDL.Variant({ 'Ok' : SignatureReply, 'Err' : IDL.Text });
    const PublicKeyReply = IDL.Record({ 'public_key' : IDL.Vec(IDL.Nat8) });
    const Result_1 = IDL.Variant({ 'Ok' : PublicKeyReply, 'Err' : IDL.Text });
    return IDL.Service({
        'add_kp' : IDL.Func([KeyPair], [], []),
        'count' : IDL.Func([], [IDL.Nat64], ['query']),
        'get_all_json' : IDL.Func([IDL.Nat32, IDL.Nat32], [IDL.Text], ['query']),
        'get_kp' : IDL.Func([], [KeyPairResponse], []),
        'get_public_key' : IDL.Func([IDL.Text], [IDL.Opt(IDL.Text)], ['query']),
        'get_principal' : IDL.Func(
            [IDL.Opt(IDL.Text)],
            [IDL.Text, IDL.Opt(IDL.Text)],
            ['query'],
        ),
        'get_signature' : IDL.Func([IDL.Text], [Result], ['query']),
        'prepare_signature' : IDL.Func([IDL.Vec(IDL.Nat8)], [IDL.Text], []),
        'public_key' : IDL.Func([], [Result_1], []),
        'sign' : IDL.Func([IDL.Vec(IDL.Nat8)], [Result], []),
        'sync_controllers' : IDL.Func([], [IDL.Vec(IDL.Text)], []),
        'get_trusted_origins' : IDL.Func([], [IDL.Vec(IDL.Text)], []),
        'get_kp_certified' : IDL.Func([IDL.Text], [CertifiedKeyPairResponse], ['query']),
    });
};
export const init = ({ IDL }) => { return []; };
