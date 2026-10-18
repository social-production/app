#!/usr/bin/env bash
# Build every zome crate (any Cargo.toml under an integrity/ or coordinator/ folder)
# to WASM with the size-optimised `zome` profile.
set -euo pipefail
cd "$(dirname "$0")/.."

args=()
while IFS= read -r manifest; do
  name=$(grep -m1 -E '^name\s*=' "$manifest" | sed -E 's/.*"(.*)".*/\1/')
  args+=(-p "$name")
done < <(find zomes dnas -name Cargo.toml \( -path '*/integrity/*' -o -path '*/coordinator/*' \))

RUSTFLAGS='--cfg getrandom_backend="custom"' \
  cargo build --profile zome --target wasm32-unknown-unknown "${args[@]}"
