#!/usr/bin/env bash
# The scenarios (scripts/check-scenarios.sh) in a cloud session: Nix inputs over git
# (scripts/cloud-nix.sh) and the no-IPv6 shim (scripts/cloud/noipv6.c), without which every
# sweettest conductor fails with "Address family not supported by protocol". Pack first:
#   bash scripts/cloud-nix.sh bash scripts/build-happ.sh
#   bash scripts/cloud-scenarios.sh [<shard> <shards>]
# Debug info is off: a full debug build fills a cloud session's disk.
set -euo pipefail
cd "$(dirname "$0")/.."

shim=$(mktemp -d)/noipv6.so
# The system compiler, so the shim loads into both system and Nix binaries.
/usr/bin/cc -shared -fPIC -o "$shim" scripts/cloud/noipv6.c
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
exec bash scripts/cloud-nix.sh env LD_PRELOAD="$shim" bash scripts/check-scenarios.sh "$@"
