#!/usr/bin/env bash
# Everything CI runs, in CI order. Run inside `nix develop`. CI runs the two halves as
# separate jobs, the scenarios in parallel shards (.github/workflows/ci.yml).
set -euo pipefail
cd "$(dirname "$0")/.."

bash scripts/check-fast.sh
bash scripts/check-scenarios.sh
