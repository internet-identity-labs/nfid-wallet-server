export const idlFactory = ({ IDL }) => {
    const Conf = IDL.Record({ 'im_canister' : IDL.Opt(IDL.Text) });
    const ICRC1 = IDL.Record({
        'ledger' : IDL.Text,
        'index' : IDL.Opt(IDL.Text),
    });
    return IDL.Service({
        'get_canisters_by_root' : IDL.Func([IDL.Text], [IDL.Vec(ICRC1)], ['query']),
        'store_icrc1_canister' : IDL.Func([IDL.Text, IDL.Opt(IDL.Text)], [], []),
    });
};
export const init = ({ IDL }) => {
    const Conf = IDL.Record({ 'im_canister' : IDL.Opt(IDL.Text) });
    return [Conf];
};
