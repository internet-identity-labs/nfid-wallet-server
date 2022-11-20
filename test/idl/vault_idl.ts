export const idlFactory = ({ IDL }) => {
  const Conf = IDL.Record({ 'ledger_canister_id' : IDL.Principal });
  const Tokens = IDL.Record({ 'e8s' : IDL.Nat64 });
  const TransferArgs = IDL.Record({
    'to_principal' : IDL.Principal,
    'to_subaccount' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'amount' : Tokens,
  });
  const Memo = IDL.Nat64;
  const TransferResult = IDL.Variant({ 'Ok' : Memo, 'Err' : IDL.Text });
  return IDL.Service({
    'sub' : IDL.Func([IDL.Text, IDL.Nat64, IDL.Nat8], [IDL.Text], ['query']),
    'transfer' : IDL.Func([TransferArgs], [TransferResult], []),
  });
};
export const init = ({ IDL }) => {
  const Conf = IDL.Record({ 'ledger_canister_id' : IDL.Principal });
  return [Conf];
};
