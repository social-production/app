#!/usr/bin/env bash
# Run a throwaway local seed node: bootstrap/relay, conductor and gateway.
# Data lives in a temp dir and is deleted on exit. Run inside `nix develop`.
# UI: VITE_SP_GATEWAYS=http://localhost:8080 npm run dev:ui
set -euo pipefail
cd "$(dirname "$0")/.."

[[ -f happ/social_production.happ ]] || bash scripts/build-happ.sh

data=$(mktemp -d)
trap 'kill $(jobs -p) 2>/dev/null || true; tail -n 20 "$data/conductor.log" >&2; rm -rf "$data"' EXIT

kitsune2-bootstrap-srv --listen 127.0.0.1:4433 >"$data/bootstrap.log" 2>&1 &

export DATA_DIR="$data" ADMIN_PORT=4444 APP_WS_PORT=8888
export BOOTSTRAP_URL=http://127.0.0.1:4433 RELAY_URL=http://127.0.0.1:4433
perl -pe 's/\$\{(\w+)\}/$ENV{$1}/g' deploy/seed-node/conductor.yaml >"$data/conductor.yaml"
# Local only: the relay is plain http here (Railway terminates TLS in production).
cat >>"$data/conductor.yaml" <<'YAML'
  advanced:
    irohTransport:
      relayAllowPlainText: true
YAML
echo "dev-passphrase" | holochain --piped -c "$data/conductor.yaml" >"$data/conductor.log" 2>&1 &

HAPP_PATH=happ/social_production.happ RUST_LOG=${RUST_LOG:-info} cargo run -q -p gateway
