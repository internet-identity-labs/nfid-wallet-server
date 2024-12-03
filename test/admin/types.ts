
export interface ICRC1CsvData {
    logo: string | undefined
    name: string
    ledger: string
    category: string
    index: string | undefined
    symbol: string
    fee: string
    decimals: string
    root_canister_id: string | undefined
    date_added: string
}

export enum CategoryCSV {
    Sns = "Sns",
    Known = "Known",
    Native = "Native",
    Spam = "Spam",
    ChainFusionTestnet = "ChainFusionTestnet",
    ChainFusion = "ChainFusion",
    Community = "Community",
}

export interface RootCanister {
    ledger_canister_id: string,
    sns_root_canister_id: string,
    ckerc20_orchestrator_id: string | null,
    ckerc20_contract: string | null,
    icrc1_metadata: {
        icrc1_fee: string,
        icrc1_name: string,
        icrc1_logo: string | null,
        icrc1_symbol: string,
        icrc1_decimals: string,
        icrc1_total_supply: string,
        icrc1_max_memo_length: string,
    }
}

export interface SwapLifecycle {
    lifecycle: string;
    decentralization_sale_open_timestamp_seconds: number;
}

export interface CanisterObjectA {
    root_canister_id: string;
    name: string;
    url: string;
    logo: string;
    description: string;
    swap_lifecycle: SwapLifecycle;
    enabled: boolean;
    nns_proposal_id_create_sns: string | null;
}

interface Icrc1MintingAccount {
    owner: string;
    subaccount: string | null;
}

interface Icrc1Metadata {
    icrc1_fee: string;
    icrc1_max_memo_length: string;
    icrc1_name: string;
    icrc1_symbol: string;
    icrc1_decimals: string;
    icrc1_total_supply: string;
}

interface NervousSystemParameters {
    reject_cost_e8s: number;
    icrc1_decimals: string;
    icrc1_total_supply: string;
    default_followees: { followees: any[] };
    transaction_fee_e8s: number;
    max_number_of_neurons: number;
    icrc1_max_memo_length: string;
    max_age_bonus_percentage: number;
    icrc1_minting_account: Icrc1MintingAccount;
}

interface NervousSystemParameters {
    reject_cost_e8s: number;
    max_proposals_to_keep_per_action: number;
    max_dissolve_delay_bonus_percentage: number;
    max_number_of_principals_per_neuron: number;
    max_number_of_proposals_with_ballots: number;
    wait_for_quiet_deadline_increase_seconds: number;
    default_followees: { followees: any[] };
    transaction_fee_e8s: number;
    neuron_minimum_dissolve_delay_to_vote_seconds: number;
}

export interface CanisterObject {
    neuron_claimer_permissions: { permissions: any[] };
    url: string;
    created_at: string;
    max_neuron_age_for_age_bonus: number;
    governance_canister_id: string;
    logo: string;
    updated_at: string;
    root_canister_id: string;
    neuron_grantable_permissions: { permissions: any[] };
    description: string;
    initial_voting_period_seconds: number;
    max_proposals_to_keep_per_action: number;
    enabled: boolean;
    ledger_canister_id: string;
    icrc1_metadata: Icrc1Metadata;
    max_dissolve_delay_bonus_percentage: number;
    icp_treasury_account: string;
    swap_canister_id: string;
    max_number_of_principals_per_neuron: number;
    max_number_of_proposals_with_ballots: number;
    nervous_system_parameters: NervousSystemParameters;
    wait_for_quiet_deadline_increase_seconds: number;
    sns_treasury_account: string;
    index_canister_id: string;
    neuron_minimum_dissolve_delay_to_vote_seconds: number;
}

