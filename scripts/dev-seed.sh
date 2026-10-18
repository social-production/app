#!/usr/bin/env bash
# Run a throwaway local seed node: bootstrap/relay + gateway (with its embedded conductor).
# Data lives in a temp dir and is deleted on exit. Run inside `nix develop`.
# UI: VITE_SP_GATEWAYS=http://localhost:8080 npm run dev:ui  (passkeys need localhost:5173)
set -euo pipefail
cd "$(dirname "$0")/.."

[[ -f happ/social_production.happ ]] || bash scripts/build-happ.sh

data=$(mktemp -d)
trap 'kill $(jobs -p) 2>/dev/null || true; rm -rf "$data"' EXIT

kitsune2-bootstrap-srv --listen 127.0.0.1:4433 >"$data/bootstrap.log" 2>&1 &

export DATA_DIR="$data" LAIR_PASSPHRASE=dev-passphrase
export BOOTSTRAP_URL=http://127.0.0.1:4433 RELAY_URL=http://127.0.0.1:4433
export HAPP_PATH=happ/social_production.happ RP_ID=localhost RP_ORIGIN=${RP_ORIGIN:-http://localhost:5173}
export POW_BITS=${POW_BITS:-16} SIGNUPS_PER_IP_PER_DAY=${SIGNUPS_PER_IP_PER_DAY:-1000}
export RUST_LOG=${RUST_LOG:-info,holochain=warn,kitsune2=warn}
cargo run -q -p gateway
