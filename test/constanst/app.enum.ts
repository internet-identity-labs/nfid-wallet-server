import canisterIds from "../../canister_ids.json";

export enum App {
  InternetIdentityTest = "internet_identity_test",
  IdentityManager = "identity_manager",
  IdentityManagerReplica = "identity_manager_replica",
  Vault = "vault",
  ECDSASigner = "signer_ic",
  UserRegistry = "user_registry",
  ICRC1Oracle = "icrc1_oracle",
  DelegationFactory = "delegation_factory",
  NFIDStorage = "nfid_storage",
  SwapTrsStorage = "swap_trs_storage",
}

export const APP_CANISTER_IDS: Record<App, string> = {
  [App.IdentityManager]: canisterIds.identity_manager.dev,
  [App.IdentityManagerReplica]: "a4gq6-oaaaa-aaaab-qaa4q-cai",
  [App.InternetIdentityTest]: canisterIds.internet_identity_test.dev,
  [App.Vault]: canisterIds.vault.dev,
  [App.ECDSASigner]: canisterIds.signer_ic.dev,
  [App.UserRegistry]: canisterIds.user_registry.dev,
  [App.ICRC1Oracle]: canisterIds.icrc1_oracle.dev,
  [App.DelegationFactory]: canisterIds.delegation_factory.dev,
  [App.NFIDStorage]: canisterIds.nfid_storage.dev,
  [App.SwapTrsStorage]: canisterIds.swap_trs_storage.dev,
};
