export const idlFactory = ({ IDL }) => {
    const KeyPair = IDL.Record({
        'public_key' : IDL.Text,
        'private_key_encrypted' : IDL.Text,
    });
    const KeyPairResponse = IDL.Record({
        'key_pair' : IDL.Opt(KeyPair),
        'princ' : IDL.Text,
    });
    return IDL.Service({
        'add_kp' : IDL.Func([KeyPair], [], []),
        'get_kp' : IDL.Func([], [KeyPairResponse]),
        'get_pk_by_principal' : IDL.Func([IDL.Text], [IDL.Text], []),
        'get_principal' : IDL.Func(
            [IDL.Opt(IDL.Text)],
            [IDL.Text, IDL.Opt(IDL.Text)],
            ['query'],
        ),
        'get_all_json' : IDL.Func([IDL.Nat32, IDL.Nat32], [IDL.Text], ['query']),
        'sync_controllers' : IDL.Func([], [IDL.Vec(IDL.Text)], []),
        'count' : IDL.Func([], [IDL.Nat64], ['query']),
    });
};
export const init = ({ IDL }) => { return []; };
