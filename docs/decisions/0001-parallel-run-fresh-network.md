# 0001. Run in parallel with FastAPI; fresh network, no import

Status: accepted (2026-10-18)

## Context
The FastAPI site (web/ + web-backend/) is live and tested weekly by Grokbot. P2P is unproven.

## Decision
Build P2P separately and keep the FastAPI site live until P2P reaches full feature parity, then replace it. The P2P network starts empty. No legacy data is ever imported.

## Consequences
No migration code. The legacy table map in DOMAIN_MODEL.md is a parity checklist only. Users re-create accounts at cutover.
