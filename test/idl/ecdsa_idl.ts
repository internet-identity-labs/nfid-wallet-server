export const idlFactory = ({ IDL }) => {
    const Conf = IDL.Record({
        'key' : IDL.Text,
        'ttl' : IDL.Nat64,
        'price' : IDL.Nat64,
    });
    const KeyPair = IDL.Record({
        'public_key' : IDL.Text,
        'private_key_encrypted' : IDL.Text,
    });
    const KeyPairResponse = IDL.Record({
        'key_pair' : IDL.Opt(KeyPair),
        'princ' : IDL.Text,
    });
    const SignatureReply = IDL.Record({ 'signature' : IDL.Vec(IDL.Nat8) });
    const Result = IDL.Variant({ 'Ok' : SignatureReply, 'Err' : IDL.Text });
    const PublicKeyReply = IDL.Record({ 'public_key' : IDL.Vec(IDL.Nat8) });
    const Result_1 = IDL.Variant({ 'Ok' : PublicKeyReply, 'Err' : IDL.Text });
    return IDL.Service({
        'add_kp' : IDL.Func([KeyPair], [], []),
        'get_kp' : IDL.Func([], [KeyPairResponse], ['query']),
        'get_principal' : IDL.Func(
            [IDL.Opt(IDL.Text)],
            [IDL.Text, IDL.Opt(IDL.Text)],
            ['query'],
        ),
        'get_signature' : IDL.Func([IDL.Text], [Result], ['query']),
        'prepare_signature' : IDL.Func([IDL.Vec(IDL.Nat8)], [IDL.Text], []),
        'public_key' : IDL.Func([], [Result_1], []),
        'sign' : IDL.Func([IDL.Vec(IDL.Nat8)], [Result], []),
    });
};
export const init = ({ IDL }) => {
    const Conf = IDL.Record({
        'key' : IDL.Text,
        'ttl' : IDL.Nat64,
        'price' : IDL.Nat64,
    });
    return [IDL.Opt(Conf)];
};
