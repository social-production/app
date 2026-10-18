# 0026. Signup protection: proof-of-work, per-IP limit, New level

Status: accepted (2026-10-18)

## Context
Each hosted signup costs the seed node a hosted agent and its cells.

## Decision
Signup and restore need a browser proof-of-work (POW_BITS, default 20) and are limited per IP per day (SIGNUPS_PER_IP_PER_DAY, default 5). New accounts start at New standing and cannot vote. No third-party captcha is used.

## Consequences
Mass signups are slow and expensive. Bots can still exist but carry no vote. Several people behind one shared IP may hit the daily limit.
