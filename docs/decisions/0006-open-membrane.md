# 0006. Open, platform-wide membrane; no per-project join rules

Status: accepted (2026-10-18)

## Context
Join rules set per project could let a group make shared means of production private.

## Decision
The membrane is fixed in integrity code. Anyone may join the directory and any project, and read everything there. Only conversations are private. Changing this requires a new DNA version via a community-approved PR.

## Consequences
Projects cannot close themselves. Sybil resistance comes from standing (ADR 0007), not gatekeeping.
