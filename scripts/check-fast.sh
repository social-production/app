#!/usr/bin/env bash
# Everything CI runs except the scenarios, in CI order. Run inside `nix develop`.
set -euo pipefail
cd "$(dirname "$0")/.."

bash scripts/check-boundaries.sh
bash scripts/check-sizes.sh

npm run check -w ui
npm run lint -w ui
npm run build -w ui

# native/ embeds ui/build and the packed hApp at compile time, so pack first.
bash scripts/build-happ.sh

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --exclude scenarios
