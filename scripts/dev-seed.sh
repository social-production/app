#!/usr/bin/env bash
# Run a throwaway local seed node: bootstrap/relay + gateway (with its embedded conductor).
# Data lives in a temp dir and is deleted on exit. Run inside `nix develop`.
# UI: VITE_SP_GATEWAYS=http://localhost:8080 npm run dev:ui  (passkeys need localhost:5173)
set -euo pipefail
cd "$(dirname "$0")/.."

[[ -f happ/social_production.happ ]] || bash scripts/build-happ.sh

data=$(mktemp -d)
trap 'kill $(jobs -p) 2>/dev/null || true; rm -rf "$data"' EXIT

# A dev copy of the hApp whose founders are the fixed test identities in
# tests/e2e/fixtures/dev-founders.json, so local tests can reach Full standing (governance).
# Production founders stay in dnas/directory/dna.yaml; this copy never leaves the temp dir.
dev="$data/dev-happ"
mkdir -p "$dev/directory" "$dev/happ"
python3 - "$dev" "$(pwd)" <<'PY'
import json, re, sys
dev, root = sys.argv[1], sys.argv[2]
founders = json.load(open(f"{root}/tests/e2e/fixtures/dev-founders.json"))
dna = open(f"{root}/dnas/directory/dna.yaml").read().replace("../../target", f"{root}/target")
block = "".join(f"      - name: {f['name']}\n        agent: {f['agent']}\n" for f in founders)
dna = re.sub(r"(    founders:\n)(      - name: .*\n        agent: .*\n)+", lambda m: m.group(1) + block, dna)
open(f"{dev}/directory/dna.yaml", "w").write(dna)
happ = open(f"{root}/happ/happ.yaml").read()
happ = happ.replace("../dnas/directory/sp_directory.dna", f"{dev}/directory/sp_directory.dna")
happ = happ.replace("../dnas/", f"{root}/dnas/")
open(f"{dev}/happ/happ.yaml", "w").write(happ)
PY
hc dna pack "$dev/directory" -o "$dev/directory/sp_directory.dna" >/dev/null
hc app pack "$dev/happ" -o "$dev/social_production.happ" >/dev/null

kitsune2-bootstrap-srv --listen 127.0.0.1:4433 >"$data/bootstrap.log" 2>&1 &

export DATA_DIR="$data" LAIR_PASSPHRASE=dev-passphrase
export BOOTSTRAP_URL=http://127.0.0.1:4433 RELAY_URL=http://127.0.0.1:4433
export HAPP_PATH=$dev/social_production.happ RP_ID=localhost RP_ORIGIN=${RP_ORIGIN:-http://localhost:5173}
export POW_BITS=${POW_BITS:-16} SIGNUPS_PER_IP_PER_DAY=${SIGNUPS_PER_IP_PER_DAY:-1000}
export RUST_LOG=${RUST_LOG:-info,holochain=warn,kitsune2=warn}
# RELEASE=1 runs the optimised build, as production does (memory measurements, ADR 0053).
cargo run ${RELEASE:+--release} -q -p gateway
