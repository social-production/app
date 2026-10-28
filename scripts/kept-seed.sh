#!/usr/bin/env bash
# scripts/dev-seed.sh with a kept data dir, for restart and upgrade experiments (ADR 0062, 0063).
# KEEP=<dir> keeps the conductor between runs; EXTRA_FOUNDER=1 renames a founder, which changes the
# directory DNA hash and so the hApp version (as an integrity change does). Cloud sessions run it as
#   KEEP=… bash scripts/cloud-nix.sh env LD_PRELOAD=<noipv6 shim> bash scripts/kept-seed.sh
set -euo pipefail
: "${KEEP:?set KEEP to a folder that survives restarts}"
cd "$(dirname "$0")/.."
data=$KEEP; mkdir -p "$data"
trap 'kill $(jobs -p) 2>/dev/null || true' EXIT
dev="$data/dev-happ"; mkdir -p "$dev/directory" "$dev/happ"
python3 - "$dev" "$(pwd)" "${EXTRA_FOUNDER:-0}" <<'PY'
import json, re, sys
dev, root, extra = sys.argv[1], sys.argv[2], sys.argv[3] == "1"
founders = json.load(open(f"{root}/tests/e2e/fixtures/dev-founders.json"))
if extra:
    founders[0]["name"] += " v2"  # a different directory hash, so a new hApp version
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
export DATA_DIR="$data/gw" LAIR_PASSPHRASE=dev-passphrase
export BOOTSTRAP_URL=http://127.0.0.1:4433 RELAY_URL=http://127.0.0.1:4433
export HAPP_PATH=$dev/social_production.happ RP_ID=localhost RP_ORIGIN=http://localhost:4174
export POW_BITS=16 SIGNUPS_PER_IP_PER_DAY=1000 RUST_LOG=info,holochain=warn,kitsune2=warn
cargo run --release --features jemalloc -q -p gateway
