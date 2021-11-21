#!/usr/bin/env bash
set -euo pipefail

II_DIR="$(dirname "$0")"
TARGET="wasm32-unknown-unknown"

cargo build --manifest-path "$II_DIR/Cargo.toml" --target $TARGET --release -j1

# keep version in sync with Dockerfile
cargo install ic-cdk-optimizer --version 0.3.1 --root "$II_DIR"/../../target
STATUS=$?

if [ "$STATUS" -eq "0" ]; then
      "$II_DIR"/../../target/bin/ic-cdk-optimizer \
      "$II_DIR/../../target/$TARGET/release/identity_manager.wasm" \
      -o "$II_DIR/../../target/$TARGET/release/identity_manager.wasm"

  true
else
  echo Could not install ic-cdk-optimizer.
  false
fi
