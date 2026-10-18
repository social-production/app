# 0009. Generic Proposal → Vote → Outcome, with special kinds allowed

Status: accepted (2026-10-18)

## Context
FastAPI has ~40 vote/request tables, one per decision type.

## Decision
One generic primitive. Each kind registers its threshold, population rule, payload validation and apply rule. A few special kinds (e.g. plan selection) hand-write their tally but still end in a validated Outcome. Any member may write an Outcome; validators recompute the tally.

## Consequences
One validated code path. New decision types are small modules. The population rule is an open M3 item (GOVERNANCE.md).
