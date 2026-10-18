#!/usr/bin/env bash
# Enforces the layer rules in docs/CODE_RULES.md. Fails CI on any violation.
set -uo pipefail
cd "$(dirname "$0")/.."

failures=0

# forbid <description> <grep -E pattern> <paths...>
forbid() {
  local description=$1 pattern=$2
  shift 2
  local hits
  hits=$(grep -rnE --include='*.ts' --include='*.svelte' --include='*.toml' "$pattern" "$@" 2>/dev/null)
  if [[ -n "$hits" ]]; then
    echo "✗ $description"
    echo "$hits" | sed 's/^/    /'
    failures=$((failures + 1))
  fi
}

ui=ui/src

# UI: only lib/holo talks to Holochain.
forbid "@holochain/client may only be imported in ui/src/lib/holo/" \
  "from ['\"]@holochain/client" "$ui/routes" "$ui/lib/features" "$ui/lib/components" \
  "$ui/lib/services" "$ui/lib/stores" "$ui/lib/domain"

# UI: routes are thin — features and components only.
forbid "routes may import only #lib/features and #lib/components" \
  "from ['\"]#lib/(holo|services|stores|domain)" "$ui/routes"

# UI: features go through services, never straight to holo.
forbid "features must call services, not #lib/holo" "from ['\"]#lib/holo" "$ui/lib/features"

# UI: services are stateless adapters over holo.
forbid "services must not import features, components or stores" \
  "from ['\"]#lib/(features|components|stores)" "$ui/lib/services"

# UI: components are presentational.
forbid "components must not import services, stores or holo" \
  "from ['\"]#lib/(holo|services|stores)" "$ui/lib/components"

# UI: domain types are leaf modules.
forbid "domain types must not import other layers" \
  "from ['\"]#lib/(holo|services|stores|features|components)" "$ui/lib/domain"

# Rust: integrity zomes use hdi only (no hdk, no coordinator crates).
forbid "integrity zomes must not depend on hdk" "^hdk\\s*=" zomes/*/integrity dnas/*/zomes/integrity

# Rust: pure crates stay pure so they test without a conductor.
forbid "crates/sp_* must not depend on Holochain" "^(hdi|hdk|holochain)\\s*=" crates

if ((failures > 0)); then
  echo "$failures boundary rule(s) broken — see docs/CODE_RULES.md"
  exit 1
fi
echo "✓ boundaries ok"
