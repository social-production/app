#!/usr/bin/env bash
# The multi-conductor scenarios (run `scripts/build-happ.sh` first). Inside `nix develop`.
#   bash scripts/check-scenarios.sh                    every scenario file
#   bash scripts/check-scenarios.sh <shard> <shards>   every <shards>-th file from <shard> on
# CI runs the shards as parallel jobs; each file (sorted by name) lands in exactly one shard,
# so a new file is picked up without touching CI.
set -euo pipefail
cd "$(dirname "$0")/.."

shard=${1:-0}
shards=${2:-1}
tests=()
i=0
for file in tests/scenarios/tests/*.rs; do
  if ((i % shards == shard)); then
    tests+=(--test "$(basename "$file" .rs)")
  fi
  i=$((i + 1))
done
if ((${#tests[@]} == 0)); then
  echo "no scenario files in shard $shard of $shards"
  exit 0
fi
echo "scenarios: ${tests[*]}"
# One at a time: on small CI runners, parallel conductors starve each other until signed zome
# calls expire (BadNonce "Expired").
cargo test -p scenarios "${tests[@]}" -- --test-threads=1
