
export interface ICRC1CsvData {
    logo: string | undefined
    name: string
    ledger: string
    category: string
    index: string | undefined
    symbol: string
    fee: string
    decimals: string
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
