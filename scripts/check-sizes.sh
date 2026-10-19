#!/usr/bin/env bash
# File length limit (docs/CODE_RULES.md): warn above 240 lines, fail above 300.
# Function length is enforced by clippy (too_many_lines = 60) and eslint (max-lines-per-function).
set -uo pipefail
cd "$(dirname "$0")/.."

max=300
warn=240
failures=0

while IFS= read -r file; do
  lines=$(wc -l <"$file")
  if ((lines > max)); then
    echo "✗ $file: $lines lines (max $max)"
    failures=$((failures + 1))
  elif ((lines > warn)); then
    echo "! $file: $lines lines (approaching $max)"
  fi
done < <(git ls-files --cached --others --exclude-standard -- '*.rs' '*.ts' '*.svelte' '*.sh' |
  grep -vE '(^|/)(generated|paraglide)/|^vendor/')

if ((failures > 0)); then
  echo "$failures file(s) over $max lines — split them by responsibility"
  exit 1
fi
echo "✓ file sizes ok"
