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

# DNA hashes are hashes of this WASM, so every machine (desktop, CI, Railway's Docker)
# must produce identical bytes: strip machine-specific paths that panic messages embed.
# CARGO_ENCODED_RUSTFLAGS (0x1f-separated) because the repo path may contain spaces.
cargo_home=${CARGO_HOME:-$HOME/.cargo}
sep=$'\x1f'
flags="--cfg${sep}getrandom_backend=\"custom\""
flags+="${sep}--remap-path-prefix=${cargo_home}=/cargo"
flags+="${sep}--remap-path-prefix=$(pwd)=/sp"
CARGO_ENCODED_RUSTFLAGS="$flags" \
  cargo build --profile zome --target wasm32-unknown-unknown "${args[@]}"
