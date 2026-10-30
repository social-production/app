#!/usr/bin/env bash
# The multi-conductor scenarios (run `scripts/build-happ.sh` first). Inside `nix develop`.
#   bash scripts/check-scenarios.sh                    every scenario file
#   bash scripts/check-scenarios.sh <shard> <shards>   one of <shards> groups of files
# CI runs the shards as parallel jobs. Files are dealt heaviest first to the lightest shard,
# using the minutes each took in CI (below), so every file lands in exactly one shard and a new
# file (counted as NEW_FILE_MINUTES until it is measured) is picked up without touching CI.
set -euo pipefail
cd "$(dirname "$0")/.."

shard=${1:-0}
shards=${2:-1}

# Minutes per file on CI's runners (PR #75, 2026-10-30). Update when a file's time changes a lot.
declare -A minutes=(
  [governance]=45 [private_events]=40 [activities]=30 [discussion]=27 [plans]=27
  [scopes]=25 [projects]=22 [identity]=21 [health]=21 [groups]=18 [events]=14
  [follows]=12 [values]=11 [challenges]=11 [links]=9 [help_requests]=9 [attachments]=9
  [messages]=8 [reverts]=7 [receipts]=6 [calls]=5 [feedback]=3 [scale]=0
)
NEW_FILE_MINUTES=15

# Heaviest first (ties by name, so every shard computes the same split).
order=$(for file in tests/scenarios/tests/*.rs; do
  name=$(basename "$file" .rs)
  echo "${minutes[$name]:-$NEW_FILE_MINUTES} $name"
done | sort -k1,1nr -k2,2)

load=()
for ((s = 0; s < shards; s++)); do load[s]=0; done
tests=()
while read -r weight name; do
  lightest=0
  for ((s = 1; s < shards; s++)); do
    if ((load[s] < load[lightest])); then lightest=$s; fi
  done
  load[lightest]=$((load[lightest] + weight))
  if ((lightest == shard)); then
    tests+=(--test "$name")
  fi
done <<<"$order"

if ((${#tests[@]} == 0)); then
  echo "no scenario files in shard $shard of $shards"
  exit 0
fi
echo "scenarios (~${load[shard]} min on CI): ${tests[*]}"
# One at a time: on small CI runners, parallel conductors starve each other until signed zome
# calls expire (BadNonce "Expired").
cargo test -p scenarios "${tests[@]}" -- --test-threads=1
