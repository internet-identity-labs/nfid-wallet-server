export const idlFactory = ({ IDL }) => {
    const Conf = IDL.Record({ 'ledger_canister_id' : IDL.Principal });
    const Participant = IDL.Record({});
    const Account = IDL.Record({
        'account_id' : IDL.Nat8,
        'name' : IDL.Text,
        'sub_account' : IDL.Vec(IDL.Nat8),
    });
    const Group = IDL.Record({
        'participants' : IDL.Vec(Participant),
        'name' : IDL.Text,
        'accounts' : IDL.Vec(Account),
        'group_id' : IDL.Nat64,
    });
    return IDL.Service({
        'register_group' : IDL.Func([IDL.Text], [Group], []),
        'sub' : IDL.Func([IDL.Text, IDL.Nat64, IDL.Nat8], [IDL.Text], ['query']),
    });
};
export const init = ({ IDL }) => {
    const Conf = IDL.Record({ 'ledger_canister_id' : IDL.Principal });
    return [Conf];
};
