#!/usr/bin/env bash
# The local seed node (scripts/dev-seed.sh) in a cloud session: Nix inputs over git
# (scripts/cloud-nix.sh), plus a shim for containers without IPv6, where Holochain's interfaces
# refuse to start (scripts/cloud/noipv6.c). Browser tests then run as usual, with
#   CHROMIUM_PATH=/opt/pw-browsers/chromium bash scripts/e2e-local.sh tests/e2e/<test>.mjs
set -euo pipefail
cd "$(dirname "$0")/.."

# dev-seed.sh deletes its data on exit, but a killed session (container restart) skips that and
# leaves ~370 MB per run under nix-shell temp dirs. With no seed node running, sweep them.
if ! curl -sf -o /dev/null http://localhost:8080/readyz; then
  for stale in /tmp/nix-shell.*/tmp.*/dev-happ; do
    if [[ -e "$stale" ]]; then rm -rf "$(dirname "$stale")"; fi
  done
fi

shim=$(mktemp -d)/noipv6.so
# The system compiler, so the shim loads into both system and Nix binaries.
/usr/bin/cc -shared -fPIC -o "$shim" scripts/cloud/noipv6.c
export RP_ORIGIN=${RP_ORIGIN:-http://localhost:4174}
exec bash scripts/cloud-nix.sh env LD_PRELOAD="$shim" bash scripts/dev-seed.sh
