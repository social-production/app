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

# Minutes per file on CI's runners, measured on PR #163 (2026-11-16; `values` was cut off there,
# so it keeps PR #158's figure plus a margin). Update when a file's time changes a lot: a shard
# over CI's limit is cancelled mid-file (shard 4 lost `values` that way twice on #163, when
# `bookings`, `convert` and `board` were still counted as new files).
declare -A minutes=(
  [governance]=49 [private_events]=43 [reports]=42 [service_requests]=38 [bookings]=33
  [activities]=32 [values]=30 [scopes]=28 [discussion]=28 [collective_bookings]=24
  [identity]=23 [projects]=22 [events]=22 [plans]=20 [help_requests]=20 [health]=18
  [threads]=16 [proposals]=15 [challenges]=15 [links]=14 [convert]=14 [board]=14 [groups]=11 [group_managers]=8 [seedless]=4
  [places]=10 [messages]=10 [chat_reports]=10 [software]=9 [bot_reviews]=9 [attachments]=8
  [reverts]=7 [follows]=7 [receipts]=6 [feedback]=5 [collective_requests]=5 [notes]=4
  [feedback_comments]=3 [calls]=3 [scale]=0
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
