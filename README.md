<div style="display:flex;flex-direction:column;">
  <a href="https://nfid.one/">
    <img src="./nfid-wallet-og.png" alt="NFID Wallet" role="presentation"/>
  </a>

<br/>
<br/>

**Wallet backend tests**
<br/>
[![GitHub Delegation Factory Tests Workflow Status](https://img.shields.io/github/actions/workflow/status/internet-identity-labs/nfid-wallet-server/delegation_factory.yml?logo=github&label=Delegation%20Factory%20Tests)](https://github.com/internet-identity-labs/nfid-wallet-server/actions/workflows/delegation_factory.yml)
[![GitHub Identity Manager Tests Workflow Status](https://img.shields.io/github/actions/workflow/status/internet-identity-labs/nfid-wallet-server/identity_manager.yml?logo=github&label=Identity%20Manager%20Tests)](https://github.com/internet-identity-labs/nfid-wallet-server/actions/workflows/identity_manager.yml)
[![GitHub NFID Storage Tests Workflow Status](https://img.shields.io/github/actions/workflow/status/internet-identity-labs/nfid-wallet-server/nfid_storage.yml?logo=github&label=NFID%20Storage%20Tests)](https://github.com/internet-identity-labs/nfid-wallet-server/actions/workflows/nfid_storage.yml)
[![GitHub Wallet Swap Tests Workflow Status](https://img.shields.io/github/actions/workflow/status/internet-identity-labs/nfid-wallet-server/swap_trs_storage.yml?logo=github&label=Wallet%20Swap%20Tests)](https://github.com/internet-identity-labs/nfid-wallet-server/actions/workflows/swap_trs_storage.yml)

**ICRC1 public token registry API tests**
<br/>
[![GitHub ICRC1 Oracle Tests Workflow Status](https://img.shields.io/github/actions/workflow/status/internet-identity-labs/nfid-wallet-server/icrc1_oracle.yml?logo=github&label=ICRC1%20Oracle%20Tests)](https://github.com/internet-identity-labs/nfid-wallet-server/actions/workflows/icrc1_oracle.yml)
[![GitHub ICRC1 Registry Tests Workflow Status](https://img.shields.io/github/actions/workflow/status/internet-identity-labs/nfid-wallet-server/icrc1_registry.yml?logo=github&label=ICRC1%20Registry%20Tests)](https://github.com/internet-identity-labs/nfid-wallet-server/actions/workflows/icrc1_registry.yml)

# NFID Wallet

NFID Wallet is the easiest to use, hardest to lose, and only wallet governed by a DAO powered by the Internet Computer Protocol's [Chain Fusion](https://internetcomputer.org/chainfusion) technology.

---

## Features

### Lowest barrier to entry

NFID Wallet takes wallet creation down from minutes to seconds with:

-   Email onboarding
-   Browser-based accessibility
-   No need for a seed phrase

Users create an NFID Wallet on any browser from any device in seconds with just an email address. Email onboarding is a managed service offered by Internet Identity Labs, and users can switch this service off by enabling "self-sovereign mode" whereby only user-owned passkeys are able to authenticate wallet access. With this mode enabled, users can bypass email authentication altogether and authenticate directly with a passkey. If users attempt to use an email address, they will be required to authenticate with a passkey anyway.

### Highest security guarantees

In contrast to centrally-controlled browser extension or mobile app wallets, NFID Wallet requires only trust in the Internet Computer Protocol:

-   Fully on-chain
-   Fault-tolerant access
-   ICP network key custody

The NFID Wallet frontend and backend code is stored on-chain and served directly into the browser environment, making both the frontend and backend tamper-resistant against malicious actors.

Seed phrases are a single source of failure that have led to the loss of trillions of dollars of crypto. Through the use of passkeys, users can set up multiple ways of authenticating to their wallet, all of which are extremely difficult to lose or steal.

NFID Wallet leverages ICP's native threshold ECDSA (and later threshold Schnorr) feature wherein the protocol handles the key-share distribution and signature generation across ICP nodes. Read more in [DFINITY's forum post](https://forum.dfinity.org/t/threshold-ecdsa-cryptography-review-by-ncc-group-third-party-security-audit-3/13853) about NCC Group's [public audit report for this feature](https://www.nccgroup.com/us/research-blog/public-report-threshold-ecdsa-cryptography-review/).

Users have extremely high security guarantees against malicious code updates, lost logins, and lost private keys.

### Future roadmap

#### SNS DAO (complete)

In the very near future, NFID Wallet plans to commit itself to DAO control through DFINITY's SNS launchpad, after which all code updates and treasury allocations will require DAO approval.

#### Dynamic swaps (complete)

With a variety of DEXs on ICP, each with different price markets and liquidity levels, users never know where they can get the best price for their swaps. NFID Wallet plans to create a system of dynamic swaps to route one swap across multiple DEXs to get the most tokens out of the swap.

#### Native staking

Web3 thrives on openness—users connect to services while carrying their reputation across ecosystems. This interoperability has driven the success of fungible tokens, NFTs, and DeFi. Without the ability to use a consistent identity across smart contracts, this innovation wouldn’t exist.

On ICP, this seamless connectivity is still underdeveloped, contributing to lower activity. Since ICP’s genesis, for example, users have wanted to showcase their commitment by proving they’ve staked ICP in an 8-year, non-dissolving neuron. However, the absence of wallet standards and a frontend-agnostic staking platform has made this impossible.

While standards now exist, a frontend-agnostic staking platform is still missing. As the gateway to the ICP ecosystem, the wallet is the natural place to integrate staking for ICP and SNS DAOs. This feature will provide developers with a critical new building block—stake verification—to enhance their applications.

#### DeFi tab

Currently, users managing liquidity in DeFi pools must visit each DEX separately. NFID Wallet’s DeFi Tab will aggregate all LP positions in one place, making liquidity management seamless and efficient.

#### Spending cap management

Dapps like KongSwap and Waterneuron request spending approvals for tokens, leading to two key user needs:

1. Adjusting the approved spending limit.
2. Revoking permissions when needed.

This feature will allow users to easily modify or revoke token spending approvals directly within the wallet.

---

## Navigating this repository

-   [What you'll find](#what-youll-find)
-   [Prerequisites](#prerequisites)
-   [Getting Started](#getting-started)
    -   [Launch Local DFX](#launch-local-dfx)
    -   [Deploy Identity Manager](#deploy-identity-manager)
    -   [Configure for Test Environment](#configure-for-test-environment)
    -   [Synchronize Controllers](#synchronize-controllers)
    -   [Run Satellite Applications](#run-satellite-applications)
-   [Integration Tests](#integration-tests)
    -   [Prerequisites](#integration-tests-prerequisites)
    -   [Run Integration Tests](#run-integration-tests)
-   [Community & Support](#community--support)

---

## What you'll find

The current repository is the canister backend codebase for NFID Wallet, where you'll find setup, deployment, and testing instructions, as well as technical information on its architecture.

---

## Prerequisites

Ensure you have the following tools installed before diving into development:

-   **Rustup** `^v1.27.1`
-   **DFX** `^v0.23.0`
-   **jq** `^1.6`

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

-   **NodeJS** `^22.10.0`
-   **Yarn** `^v1.22.22`

### Run Integration Tests

To run the integration tests, use:

```bash
npm i && npm run test
```

---

## Community & Support

Join the NFID Wallet community to ask questions, get support, and stay updated!

-   [Discord](https://discord.gg/a9BFNrYJ99)
-   [OpenChat](https://oc.app/community/66hym-7iaaa-aaaaf-bm7aa-cai/channel/1241143482/?ref=prkg5-paaaa-aaaaf-aqbia-cai)

> **Web3 is all about community!** Let’s build, support, and grow together 🚀
