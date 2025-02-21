#!/bin/bash

# Rebuild device index command
echo "[INFO] Rebuild device index." >&2

# Use dfx identity use ic_admin for ic and dev_admin for the rest.
NETWORK_NAME=dev
CANISTER_NAME=identity_manager

# Rebuild Device Index based on Accounts command
echo "[DEBUG] dfx canister call --network '${NETWORK_NAME}' '${CANISTER_NAME}' rebuild_index" >&2
if ! result=$(dfx canister call --network "${NETWORK_NAME}" "${CANISTER_NAME}" rebuild_index); then
  echo "[ERROR] Rebuild index failed ${result}" >&2
  exit 1
fi

echo "[SUCCESS] ${result}" >&2

echo "[DEBUG] dfx canister call --network '${NETWORK_NAME}' '${CANISTER_NAME}' save_temp_stack_to_rebuild_device_index" >&2
if ! result=$(dfx canister call --network "${NETWORK_NAME}" "${CANISTER_NAME}" save_temp_stack_to_rebuild_device_index); then
  echo "[WARN] The operation of saving device index data to temp stack has been failed: ${result}" >&2
fi

echo "[SUCCESS] ${result}" >&2

remaining_amount=1
cycles_limit=180
cycle=1

while [ ${remaining_amount} -gt 0 ] && [ ${cycle} -lt ${cycles_limit} ]; do
  echo "[INFO] Cycle '${cycle}' of '${cycles_limit}'" >&2
  
  result=''

  echo "[DEBUG] dfx canister call --network '${NETWORK_NAME}' '${CANISTER_NAME}' get_remaining_size_after_rebuild_device_index_slice_from_temp_stack" >&2
  if ! result=$(dfx canister call --network "${NETWORK_NAME}" "${CANISTER_NAME}" get_remaining_size_after_rebuild_device_index_slice_from_temp_stack) || [ -z "${result}" ]; then
    echo "[WARN] The operation of rebuilding device index slice has been failed: ${result}" >&2
    break
  fi

  echo "[DEBUG] ${result}" >&2

  # Cleaning result output
  cleaned_string="$(echo ${result#(} | cut -d' ' -f 1)"
  cleaned_string="${cleaned_string//_/}"
  remaining_amount=$((cleaned_string))

  echo "[INFO] Index rebuild is in progress, amount of remaining entries: '${remaining_amount}'" >&2
  cycle=$((cycle + 1))
done

if [ ${remaining_amount} -gt 0 ]; then
  echo "[ERROR] Index rebuild is not finished, amount of remaining entries: '${remaining_amount}'" >&2
  exit 1
fi

echo "[SUCCESS] Index rebuild has been completed." >&2
