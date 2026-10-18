# 0008. Vote eligibility: member + full standing before the proposal

Status: accepted (2026-10-18)

## Context
A minimum membership time would block voting on brand-new projects.

## Decision
A vote counts if the voter is a project member (joining is instant) and their full StandingGrant predates the proposal. Only Full votes count; Warm no longer counts half.

## Consequences
New projects can decide at once. Accounts made after a proposal opens can't swing it. Diverges from FastAPI's warm=half weight.
