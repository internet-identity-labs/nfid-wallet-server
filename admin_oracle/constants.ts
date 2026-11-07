import { JsonnableEd25519KeyIdentity } from "@icp-sdk/core/identity";
import { Principal } from "@icp-sdk/core/principal";
import { MarketplaceConfig, PromotionConfig } from "../test/idl/icrc1_oracle";

// ORACLE_CANISTER_ID overrides the target, e.g. a local deployment.
export const CANISTER_ID = process.env.ORACLE_CANISTER_ID ?? "ys266-uyaaa-aaaal-ajs4q-cai";
export const KEY_PAIR: JsonnableEd25519KeyIdentity = [
    "302a300506032b6570032100772960be43f58c228f3f4e9d37df8d22323650bac6ed0781508692fc868b93e5",
    "9acc36bb4bef0730de0fa5e2b4025f729f9b1b4fb6bebef5e700b75026a1b4f3772960be43f58c228f3f4e9d37df8d22323650bac6ed0781508692fc868b93e5",
];
export const FILE_PATH = "icrc1.csv";
export const FILE_PATH_NEURON = "neurons.csv";
export const FILE_PATH_DISCOVERY = "discovery_apps.csv";
export const FILE_PATH_NFT_COLLECTIONS = "nft_collections.csv";

// NFIDW ledger + treasury principal that receives bid payments.
const NFIDW_LEDGER = "mih44-vaaaa-aaaaq-aaekq-cai";
const PROMOTION_TREASURY = "mpg2i-yyaaa-aaaaq-aaeka-cai";

const E8S = 10n ** 8n;
const NS_PER_SECOND = 1_000_000_000n;
const NS_PER_HOUR = 3_600n * NS_PER_SECOND;
const NS_PER_DAY = 24n * NS_PER_HOUR;

export type PromotionEnv = "prod" | "dev";

export const PROMOTION_CONFIG: Record<PromotionEnv, PromotionConfig> = {
    prod: {
        min_bid_e8s: 100_000n * E8S,
        bid_increment_e8s: 100n * E8S,
        locked_period_ns: 7n * NS_PER_DAY,
        feature_duration_ns: 30n * NS_PER_DAY,
        ledger_canister: Principal.fromText(NFIDW_LEDGER),
        treasury: Principal.fromText(PROMOTION_TREASURY),
    },
    dev: {
        min_bid_e8s: 5n * E8S,
        bid_increment_e8s: 1n * E8S,
        locked_period_ns: 1n * NS_PER_HOUR,
        feature_duration_ns: 1n * NS_PER_DAY,
        ledger_canister: Principal.fromText(NFIDW_LEDGER),
        treasury: Principal.fromText(PROMOTION_TREASURY),
    },
};

// ── ICRC-7 marketplace ──────────────────────────────────────────────────

const ICP_LEDGER = "ryjl3-tyaaa-aaaaa-aaaba-cai";

export type MarketplaceEnv = "prod" | "dev";

export const MARKETPLACE_CONFIG: Record<MarketplaceEnv, MarketplaceConfig> = {
    prod: {
        icp_ledger: Principal.fromText(ICP_LEDGER),
        min_price_e8s: E8S / 100n, // 0.01 ICP
        max_retries: 5,
    },
    dev: {
        icp_ledger: Principal.fromText(ICP_LEDGER),
        min_price_e8s: E8S / 1000n, // 0.001 ICP
        max_retries: 5,
    },
};
