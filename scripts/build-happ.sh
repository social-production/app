#!/usr/bin/env bash
# Build zomes, pack the three DNAs and the hApp. Add `--web` to also pack the web hApp
# (requires `npm run build -w ui` first; produces happ/social_production.webhapp).
set -euo pipefail
cd "$(dirname "$0")/.."

bash scripts/build-zomes.sh

for dna in directory project conversation; do
  hc dna pack "dnas/$dna" -o "dnas/$dna/sp_$dna.dna"
done
hc app pack happ -o happ/social_production.happ

if [[ "${1:-}" == "--web" ]]; then
  (cd ui/build && rm -f ../ui.zip && zip -qr ../ui.zip .)
  hc web-app pack happ -o happ/social_production.webhapp
fi
