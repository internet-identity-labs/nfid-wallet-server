#!/usr/bin/env bash
set -euo pipefail

if ! command -v ic-wasm; then
  echo "could not find ic-wasm" >&2
  cargo install ic-wasm
fi

REPO_DIR="$(dirname "$0")"
TARGET="wasm32-unknown-unknown"

cargo_build_args=(
  --manifest-path "$REPO_DIR/Cargo.toml"
  --target "$TARGET"
  --release
  -j1
)

echo Running cargo build "${cargo_build_args[@]}"

cargo build "${cargo_build_args[@]}"

CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_DIR/../../target/}"

echo Repo dir ""${REPO_DIR}""

echo Target dir "${CARGO_TARGET_DIR}"

ic-wasm\
  "$CARGO_TARGET_DIR/$TARGET/release/identity_manager.wasm" \
  -o "$REPO_DIR/../../identity_manager.wasm" shrink

echo DONE