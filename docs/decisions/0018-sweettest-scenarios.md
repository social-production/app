# 0018. Scenario tests use Holochain sweettest (Rust), not Tryorama

Status: accepted (2026-10-18)

## Context
Tryorama has no 0.7 release yet.

## Decision
tests/scenarios is a Rust crate using holochain::sweettest with two or more conductors.

## Consequences
One fewer language in the test stack. Tests run with cargo.
