export const idlFactory = ({ IDL }) => {
    return IDL.Service({
        'get_canisters_by_root' : IDL.Func(
            [IDL.Text],
            [IDL.Vec(IDL.Text)],
            ['query'],
        ),
        'store_icrc1_canister' : IDL.Func([IDL.Text], [], []),
    });
};
export const init = ({ IDL }) => { return []; };
