export const idlFactory = ({ IDL }: any) => {
    const ConfigurationResponse = IDL.Record({
      'env' : IDL.Opt(IDL.Text),
      'whitelisted_phone_numbers' : IDL.Opt(IDL.Vec(IDL.Text)),
      'backup_canister_id' : IDL.Opt(IDL.Text),
      'ii_canister_id' : IDL.Opt(IDL.Principal),
      'whitelisted_canisters' : IDL.Opt(IDL.Vec(IDL.Principal)),
      'git_branch' : IDL.Opt(IDL.Text),
      'lambda' : IDL.Opt(IDL.Principal),
      'token_refresh_ttl' : IDL.Opt(IDL.Nat64),
      'heartbeat' : IDL.Opt(IDL.Nat32),
      'token_ttl' : IDL.Opt(IDL.Nat64),
      'commit_hash' : IDL.Opt(IDL.Text),
    });
    const ConfigurationRequest = IDL.Record({
      'env' : IDL.Opt(IDL.Text),
      'whitelisted_phone_numbers' : IDL.Opt(IDL.Vec(IDL.Text)),
      'backup_canister_id' : IDL.Opt(IDL.Text),
      'ii_canister_id' : IDL.Opt(IDL.Principal),
      'whitelisted_canisters' : IDL.Opt(IDL.Vec(IDL.Principal)),
      'git_branch' : IDL.Opt(IDL.Text),
      'lambda' : IDL.Opt(IDL.Principal),
      'token_refresh_ttl' : IDL.Opt(IDL.Nat64),
      'heartbeat' : IDL.Opt(IDL.Nat32),
      'token_ttl' : IDL.Opt(IDL.Nat64),
      'commit_hash' : IDL.Opt(IDL.Text),
    });
    return IDL.Service({
      'get_config' : IDL.Func([], [ConfigurationResponse], []),
      'configure' : IDL.Func([ConfigurationRequest], [], []),
    });
  };
  
