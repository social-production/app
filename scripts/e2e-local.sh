#!/usr/bin/env bash
# Build the UI against the local seed node, serve it on :4174, run browser tests, stop.
# Needs `RP_ORIGIN=http://localhost:4174 bash scripts/dev-seed.sh` running (another terminal).
# usage: bash scripts/e2e-local.sh tests/e2e/accounts.mjs [tests/e2e/projects.mjs ...]
set -euo pipefail
cd "$(dirname "$0")/.."

port=4174
curl -sf -o /dev/null http://localhost:8080/readyz || { echo "local seed node isn't running"; exit 1; }
VITE_SP_GATEWAYS=http://localhost:8080 npm run build -w ui >/dev/null

(cd ui && exec npx vite preview --port "$port" --strictPort) >/dev/null 2>&1 &
preview=$!
trap 'kill "$preview" 2>/dev/null || true' EXIT
until curl -sf -o /dev/null "http://localhost:$port"; do sleep 0.5; done

for test in "$@"; do
  echo "── $test"
  node "$test" "http://localhost:$port" "${E2E_OUT:-screens}"
done
