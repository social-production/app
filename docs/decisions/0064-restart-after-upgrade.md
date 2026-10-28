# 0064. Restart once after removing old apps; projects stay hot for now

Status: **accepted** (2026-10-28). Follows [0053](./0053-wasm-module-leak.md),
[0060](./0060-jemalloc-on-the-seed-node.md) and [0062](./0062-sqlite-readers-per-database.md).

## Context
After PR #62 (a network reset) the live seed node idled at ~1.51 GB, double the ~0.7 GB after
PR #61. PR #63 changed only the gateway, so its deploy was a plain restart: idle came back at
**~0.89 GB** (`/readyz`, 8 CPUs, 88 threads). So ~0.6 GB came from how the gateway starts a new
hApp version: the conductor starts every installed app, including the old version's, before the
gateway uninstalls them (`remove_stale_apps`), and their compiled zomes stay loaded for the
life of the process (ADR 0053: wasmer never frees engine code; Holochain's `RibosomeStore` has
no remove).

The owner also asked whether projects need to run hot at all. Measured locally (release,
jemalloc, 4 CPUs, kept data dir via `scripts/kept-seed.sh`, 10 projects, 31 cells):

| Same data | Idle | Threads |
|---|---|---|
| Fresh start, empty | 714 MB | 46 |
| Plain restart, 10 projects (hot, as today) | ~527 MB | ~150 |
| Same, every app disabled (cold) | ~491 MB | 17 |
| New hApp version, same wasm, 3 old apps removed | 618 MB | 46 |

At this size a hot project costs ~3.5 MB at idle (more while busy: ~7 MB each with ADR 0062's
reader cap). Most of the idle memory is fixed: the conductor, compiled zomes, the keystore.
ADR 0053 likewise found that disabling idle hosted apps freed nothing.

## Decision
- When `remove_stale_apps` removes any app at startup, the gateway shuts the conductor down and
  exits with an error; Railway's `ON_FAILURE` policy restarts it, and the second start removes
  nothing, so the old version's code is never resident. `RESTART_AFTER_UPGRADE=0` turns it off.
  The seed node's health-check timeout goes from 120 s to 300 s to cover the extra start.
- Projects stay hot for now. Cold projects are the next step when project counts grow, in this
  order:
  1. **Index on change, not by polling:** today the archivist reads every project every 15 s
     (`ARCHIVE_EVERY`). Re-read a project when a write through the gateway marks it dirty, plus
     a slow sweep for writes from desktop peers; keep the cards on disk.
  2. **Cold archivist clones:** disable the archivist's clone of a project nobody has opened for
     some hours (`DisableCloneCell`), wake it on open or guest read, and wake a few per round to
     catch up with desktop peers. Cost: a few seconds on first open, and a cold project isn't
     served to desktop-only peers while all its members are offline.
  3. Not putting signed-out people's apps to sleep: no memory gain (ADR 0053), and they would
     stop answering calls and serving their chats.

## Consequences
- Each deploy that changes the hApp version costs one extra restart (seconds).
- If the restart loops (an old app that can't be removed), the logs show the same line each
  start; set `RESTART_AFTER_UPGRADE=0` on Railway to stop it.
- The live idle (~0.89 GB) is still above local (~0.5–0.6 GB): more accounts and an 8-CPU
  container (more tokio workers and jemalloc arenas). Re-measure after the next reset.
