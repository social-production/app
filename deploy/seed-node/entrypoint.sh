#!/usr/bin/env bash
# Start the conductor, then the gateway in the foreground. If either exits, the
# container exits and Railway restarts it.
set -euo pipefail

: "${LAIR_PASSPHRASE:?set LAIR_PASSPHRASE on the Railway service}"
: "${BOOTSTRAP_URL:?set BOOTSTRAP_URL (the bootstrap service URL)}"
: "${RELAY_URL:=$BOOTSTRAP_URL}"
export DATA_DIR ADMIN_PORT="${ADMIN_PORT:-4444}" BOOTSTRAP_URL RELAY_URL

mkdir -p "$DATA_DIR"
config="$DATA_DIR/conductor.yaml"
# Substitute ${VARS} without needing envsubst in the slim image.
perl -pe 's/\$\{(\w+)\}/$ENV{$1}/g' /app/conductor.yaml >"$config"

echo "$LAIR_PASSPHRASE" | holochain --piped -c "$config" &
conductor=$!

gateway &
gateway=$!

wait -n "$conductor" "$gateway"
echo "a seed-node process exited; stopping container" >&2
kill "$conductor" "$gateway" 2>/dev/null || true
exit 1
