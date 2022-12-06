import { call, execute } from "../util/call.util"

export const DFX = {
    STOP: () => execute(`dfx stop`),
    REMOVE_DFX_FOLDER: () => execute(`rm -rf .dfx`),
    CREATE_TEST_PERSON: () => execute(`dfx identity new test`),
    USE_TEST_ADMIN: () => execute(`dfx identity use test_admin`),
    GET_PRINCIPAL: () => call(`dfx identity get-principal`),
    INIT: () => execute(`dfx start --clean --background`),
    DEPLOY: (x: string) => execute(`echo "yes" | dfx deploy ${x} --no-wallet`),
    GET_CANISTER_ID: (x: string) => call(`dfx canister id ${x}`),
    ADD_CONTROLLER: (x: string, y: string) => execute(`dfx canister update-settings --add-controller "${x}" ${y}`),
    SYNC_CONTROLLER: () => execute(`dfx canister call identity_manager sync_controllers`),
    DEPLOY_II: () => execute(`dfx deploy internet_identity_test --no-wallet --argument '(null)'`),
    INIT_SALT: () => call(`dfx canister call internet_identity_test init_salt`),
    CONFIGURE: () => call(`dfx canister call identity_manager configure '(record {env = opt "test"})'`),
    CONFIGURE_IM: (x: string) => call(`dfx canister call identity_manager configure '(record {${x}})'`),
    CONFIGURE_REPLICA: (x: string) => call(`dfx canister call identity_manager_replica configure '(record {env = opt "test"; whitelisted_canisters = opt vec { principal "${x}" }})'`),
    CREATE_ACCOUNT: (x: string) => call(`dfx canister call identity_manager create_account '( record { anchor = ${x} })'`),
    CREATE_ACCOUNT_2: () => call(`dfx canister call identity_manager create_account '( record {name = "TEST_USER"; anchor = 12345})'`),
    CREATE_ACCOUNT_FULL: () => call(`dfx canister call identity_manager create_account '( record {name = "TEST_USER"; anchor = 12345; phone_number = "1234567";  token = "1234"})'`),
    GET_ACCOUNT: (x: string) => call(`dfx canister call ${x} get_account`),
    GET_ACCOUNT_BY_PRINCIPAL: (x: string, y: string) => call(`dfx canister call ${x} get_account_by_principal '("${y}")'`),
    GET_PN_SHA2: (x: string, y: string) => call(`dfx canister call ${x} certify_phone_number_sha2 '("${y}", "domain")'`),
    UPDATE_ACCOUNT_NAME: () => call(`dfx canister call identity_manager update_account '( record {name = opt "TEST_USER_UPDATED";})'`),
    TOKEN: (x: string, y: string, z: string, d: string) => call(`dfx canister call identity_manager post_token '(record { phone_number_encrypted = "${x}"; phone_number_hash = "${y}"; token = "${z}"; principal_id = "${d}"})'`),
    RECOVER_ACCOUNT: () => call(`dfx canister call identity_manager recover_account '(12_345:nat64)'`), 
    REMOVE_ACCOUNT: (x: string) => call(`dfx canister call ${x} remove_account`), 
    RESTORE_ACCOUNT: (x: string, y: string) => call(`dfx canister call ${x} restore_accounts '("${y}")'`),
    CONFIGURE_ESS: () => call(`dfx canister call eth_secret_storage configure '()'`), 
    SECRET_BY_SIGNATURE: (x: string) => call(`dfx canister call eth_secret_storage secret_by_signature '( \"${x}\" )'`),
}