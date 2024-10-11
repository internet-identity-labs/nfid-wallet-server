# NFID Wallet Server

#### Prerequisites

Ensure that the required items are installed prior to setting up the development environment:
- Rustup ^v1.27.1
- DFX ^v0.22.0

#### Launch local DFX

```bash
dfx start --background --clean
```

#### Deploy Identity Manger

```bash
dfx deploy identity_manager --no-wallet --specified-id "74gpt-tiaaa-aaaak-aacaa-cai"
```

#### Configure it for the test envrionemnt

```bash
dfx canister call identity_manager configure '(record {env = opt "test"})'
```

#### Synchronize its controllers

```bash
dfx canister call identity_manager sync_controllers
```

#### Run satellite apps

```bash
dfx deploy icrc1_registry  --argument '( record { im_canister = opt "74gpt-tiaaa-aaaak-aacaa-cai" })'
dfx deploy icrc1_oracle  --argument '(opt record { im_canister = opt principal "74gpt-tiaaa-aaaak-aacaa-cai" })'
dfx deploy signer_ic  --argument '(opt record { im_canister = principal "74gpt-tiaaa-aaaak-aacaa-cai" })'
dfx deploy delegation_factory  --argument '(opt record { im_canister = principal "74gpt-tiaaa-aaaak-aacaa-cai" })'
dfx deploy nfid_storage  --argument '(opt record { im_canister = principal "74gpt-tiaaa-aaaak-aacaa-cai" })'
dfx deploy swap_trs_storage  --argument '(opt record { im_canister = principal "74gpt-tiaaa-aaaak-aacaa-cai" })'
```

## Integration tests

#### Prerequisites

Make sure the following are installed before setting up the development environment:
- NodeJS ^v20.16.0
- Yarn ^v1.22.22

#### Run integration tests

```bash
npm i && npm run test
```