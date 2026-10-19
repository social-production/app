#!/usr/bin/env bash
# `nix develop` for cloud sessions whose proxy blocks GitHub tarball downloads: fetches every
# GitHub flake input over git at its locked revision instead. Same shell as
#   nix develop --accept-flake-config --command "$@"
# usage: bash scripts/cloud-nix.sh <command> [args...]
set -euo pipefail
cd "$(dirname "$0")/.."

mapfile -t overrides < <(python3 - <<'PY'
import json
lock = json.load(open("flake.lock"))
nodes = lock["nodes"]

def walk(node, path):
    for name, ref in nodes[node].get("inputs", {}).items():
        if isinstance(ref, list):  # follows another input
            continue
        locked = nodes[ref].get("locked", {})
        if locked.get("type") == "github":
            print("/".join(path + [name]))
            print(f"git+https://github.com/{locked['owner']}/{locked['repo']}"
                  f"?rev={locked['rev']}&shallow=1")
        walk(ref, path + [name])

walk(lock["root"], [])
PY
)

args=()
for ((i = 0; i < ${#overrides[@]}; i += 2)); do
  args+=(--override-input "${overrides[i]}" "${overrides[i + 1]}")
done
exec nix develop --accept-flake-config --no-write-lock-file "${args[@]}" --command "$@"
