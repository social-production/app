# 0007. Standing via cited evidence

Status: accepted (2026-10-18)

## Context
A P2P network cannot count vouches live: nodes see data at different times.

## Decision
A StandingGrant cites the exact Vouch / service-completion / activity records it relies on. Validators must_get_valid_record each one and check it. Bot reviews pause standing through a validated Outcome.

## Consequences
Same verdict on every node. Standing changes are explicit records, which makes them auditable.
