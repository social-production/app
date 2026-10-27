# 0060. jemalloc as the gateway's allocator; test founders on the test network

Status: **accepted** (2026-10-27).

## Context
With no users the seed node sat at ~1.1 GB (`/readyz` `rssMb`), and grew past 1.4 GB after a
few browser test files. Measured locally on a release seed node (`RELEASE=1`, idle, only the
archivist installed):

| Allocator | Idle | After accounts, projects, events tests |
|---|---|---|
| glibc malloc (before) | 1148 MB | (1.4 GB in ADR 0053's runs) |
| glibc, `MALLOC_ARENA_MAX=2` | 685 MB | — |
| jemalloc (preloaded, profiling on) | 491 MB | — |
| jemalloc as `#[global_allocator]` | 675 MB | 740–780 MB |

So most of the idle gigabyte was glibc's per-thread arenas holding freed memory, not live data
(ADR 0053 found the one real leak). Holochain runs many threads, each with its own arena.

Separately, browser tests that need Full standing (governance, events, chat, links,
notifications) could only run locally, because the live founders are the owner's accounts.

## Decision
- The gateway binary uses jemalloc (`tikv-jemallocator`) as its global allocator.
- The two test identities in `tests/e2e/fixtures/dev-founders.json` are also founders of the
  test network (`dnas/directory/dna.yaml`), so every browser test runs against the live site.
  Their phrases are public, so anyone can use them on the test network; the real launch uses
  fresh founder keys and drops them.
- Creating a project or chat cell retries "database is locked" (SQLite 517) a few times
  (`ui/src/lib/holo/cells.ts`): it happened when the archivist joined a new project while
  someone created one, and Holochain doesn't retry it. This was the "project creation hangs
  after earlier tests" seen in local runs.

## Consequences
- Changing the founders changes the directory DNA hash: one more directory reset.
- Re-measure after Holochain upgrades or allocator changes with the table above.
