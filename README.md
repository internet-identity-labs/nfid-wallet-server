# NFID Wallet

Welcome to the first-ever DAO-controlled, decentralized, Web3 wallet server repository. NFID Wallet DAO is on a mission to make Web3 accessible to everyone by championing ICP as the entry point and NFID Wallet as the gateway to the decentralized internet.

---

## Table of Contents
- [About](#about)
- [Prerequisites](#prerequisites)
- [Getting Started](#getting-started)
  - [Launch Local DFX](#launch-local-dfx)
  - [Deploy Identity Manager](#deploy-identity-manager)
  - [Configure for Test Environment](#configure-for-test-environment)
  - [Synchronize Controllers](#synchronize-controllers)
  - [Run Satellite Applications](#run-satellite-applications)
- [Integration Tests](#integration-tests)
  - [Prerequisites](#integration-tests-prerequisites)
  - [Run Integration Tests](#run-integration-tests)
- [Community & Support](#community--support)

---

## About
The current repository serves as the backend powerhouse of the NFID Wallet ecosystem, a DAO-managed, decentralized, Web3 wallet designed to make Web3 accessible to everyone. By connecting with Internet Computer, it acts as the trusted gateway for users to interact with decentralized finance, identity management, and more.

---

## Prerequisites

Ensure you have the following tools installed before diving into development:

- **Rustup** `^v1.27.1`
- **DFX** `^v0.22.0`
- **jq** `^1.6`

> ⚠️ Note: These versions are specific for compatibility with the Internet Computer SDK.

---

## Getting Started

### Launch Local DFX

Begin by starting a local DFX instance:

```bash
dfx start --background --clean
```

### Deploy Identity Manager

To deploy the Identity Manager canister with the specified ID, run:

```bash
dfx deploy identity_manager --no-wallet --specified-id "74gpt-tiaaa-aaaak-aacaa-cai"
```

### Configure for Test Environment

Configure the test environment by executing:

```bash
dfx canister call identity_manager configure '(record {env = opt "test"})'
```

### Synchronize Controllers

To sync controllers, use:

```bash
dfx canister call identity_manager sync_controllers
```

### Run Satellite Applications

Deploy additional canisters with these commands:

```bash
dfx deploy icrc1_registry  --argument '( record { im_canister = opt "74gpt-tiaaa-aaaak-aacaa-cai" })'
dfx deploy icrc1_oracle  --argument '(opt record { im_canister = opt principal "74gpt-tiaaa-aaaak-aacaa-cai" })'
dfx deploy signer_ic  --argument '(opt record { im_canister = principal "74gpt-tiaaa-aaaak-aacaa-cai" })'
dfx deploy delegation_factory  --argument '(opt record { im_canister = principal "74gpt-tiaaa-aaaak-aacaa-cai" })'
dfx deploy nfid_storage  --argument '(opt record { im_canister = principal "74gpt-tiaaa-aaaak-aacaa-cai" })'
dfx deploy swap_trs_storage  --argument '(opt record { im_canister = principal "74gpt-tiaaa-aaaak-aacaa-cai" })'
```

---

## Integration Tests

### Prerequisites

Install these dependencies before testing:

- **NodeJS** `^v20.16.0`
- **Yarn** `^v1.22.22`

### Run Integration Tests

To run the integration tests, use:

```bash
npm i && npm run test
```

---

## Community & Support

Join the NFID Wallet community to ask questions, get support, and stay updated!

- [Discord](https://discord.gg/a9BFNrYJ99)
- [OpenChat](https://oc.app/community/66hym-7iaaa-aaaaf-bm7aa-cai/channel/1241143482/?ref=prkg5-paaaa-aaaaf-aqbia-cai)

> **Web3 is all about community!** Let’s build, support, and grow together 🚀
