#!/usr/bin/env bash
set -euo pipefail


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

ic-wasm\
  "$CARGO_TARGET_DIR/$TARGET/release/ecdsa_signer.wasm" \
  -o "$REPO_DIR/../../ecdsa_signer.wasm" shrink
