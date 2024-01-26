export const idlFactory = ({ IDL }) => {
    return IDL.Service({
        'add_icrc1_canister' : IDL.Func([IDL.Text], [], []),
        'get_canisters' : IDL.Func([], [IDL.Vec(IDL.Text)], ['query']),
    });
};
export const init = ({ IDL }) => { return []; };
