# identity_manager

Welcome!

## Running the project locally

If you want to test project locally, you can use the following commands:

```bash
dfx start --background --clean --emulator

dfx deploy identity_manager --no-wallet

dfx canister call identity_manager configure '(record {lambda = principal "sculj-2sjuf-dxqlm-dcv5y-hin5x-zfyvr-tzngf-bt5b5-dwhcc-zbsqf-rae"; token_ttl = 60;  token_refresh_ttl = 60; env = opt "test"})'
```
where lambda is principal_id of user who will mimique CLI calls as SMS-SENDER-SERVERLESS


# pub_sub_channel

```bash
dfx deploy pub_sub_channel --no-wallet
```
