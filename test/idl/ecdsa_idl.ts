export const idlFactory = ({ IDL }) => {
    const PublicKeyReply = IDL.Record({ 'public_key' : IDL.Vec(IDL.Nat8) });
    const Result = IDL.Variant({ 'Ok' : PublicKeyReply, 'Err' : IDL.Text });
    const RawSignature = IDL.Record({
        'r' : IDL.Vec(IDL.Nat64),
        's' : IDL.Vec(IDL.Nat64),
        'v' : IDL.Nat64,
    });
    const SignatureReply = IDL.Record({
        'signature' : IDL.Vec(IDL.Nat8),
        'hex_signature' : IDL.Text,
        'raw_signature' : RawSignature,
    });
    const Result_1 = IDL.Variant({ 'Ok' : SignatureReply, 'Err' : IDL.Text });
    return IDL.Service({
        'public_key' : IDL.Func([], [Result], []),
        'sign' : IDL.Func([IDL.Vec(IDL.Nat8)], [Result_1], []),
    });
};
export const init = ({ IDL }) => { return []; };
