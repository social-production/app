# 0016. Cleanliness is enforced by CI

Status: accepted (2026-10-18)

## Context
The old repos drifted into spaghetti because nothing enforced their rules.

## Decision
Boundary checks (scripts/check-boundaries.sh), file ≤300 / function ≤60 line limits, clippy pedantic -D warnings, rustfmt, eslint + prettier, TS strict, and this ADR log.

## Consequences
Some friction when writing code; far less when reading it.
