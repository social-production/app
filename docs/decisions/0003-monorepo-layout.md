# 0003. Repo layout

Status: accepted (2026-10-18)

## Context
Clean navigation was a stated goal.

## Decision
Use the layout in ARCHITECTURE.md: crates/ (pure), zomes/ (shared zomes), dnas/<dna>/ (DNA-specific zomes + dna.yaml), happ/, services/gateway, ui/, native/, deploy/, tests/, scripts/.

## Consequences
New code has one obvious home. check-boundaries.sh relies on these paths.
