export const idlFactory = ({ IDL }) => {
    const Conf = IDL.Record({ 'key' : IDL.Text, 'price' : IDL.Nat64 });
    const PublicKeyReply = IDL.Record({ 'public_key' : IDL.Vec(IDL.Nat8) });
    const Result = IDL.Variant({ 'Ok' : PublicKeyReply, 'Err' : IDL.Text });
    const SignatureReply = IDL.Record({ 'signature' : IDL.Vec(IDL.Nat8) });
    const Result_1 = IDL.Variant({ 'Ok' : SignatureReply, 'Err' : IDL.Text });
    return IDL.Service({
        'public_key' : IDL.Func([], [Result], []),
        'sign' : IDL.Func([IDL.Vec(IDL.Nat8)], [Result_1], []),
    });
};
export const init = ({ IDL }) => {
    const Conf = IDL.Record({ 'key' : IDL.Text, 'price' : IDL.Nat64 });
    return [IDL.Opt(Conf)];
};
