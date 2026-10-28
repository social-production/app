# 0062. Cap SQLite readers per database on the seed node (memory per project)

Status: **accepted** (2026-10-28). Follows [0060](./0060-jemalloc-on-the-seed-node.md).

## Context
After jemalloc (ADR 0060) the live seed node idled at ~0.7 GB but still grew with projects:
all 18 browser test files took it from 698 to 1407 MB, in steps that came with new projects.

Every project is its own DNA, so the conductor opens a *space* per project with two SQLite
databases (`dht-<hash>.db` and `p2p-peer-meta-<hash>.db`). `holochain_data` gives each a pool
of `db_max_readers + 1` connections (idle ones close after 30 s), and **sqlx runs every
connection on its own OS thread with its own SQLite page cache**. Holochain's default
`db_max_readers` is 2 × CPUs, at least 8, so a container that reports many CPUs pays more per
project. The archivist reads every project each minute and gossip touches them, so the pools
are rarely idle for long.

Measured locally (release build, jemalloc, 4 CPUs, `tests/e2e/memory-projects.mjs`: founder
one creates 6 projects, `/readyz` read once the archivist has joined each; threads from
`/proc/<pid>/task`):

| `db_max_readers` | Connections per DB | After project 1 → 6 | Per project | Peak threads |
|---|---|---|---|---|
| default (8 here) | 9 | 659 → 737 MB | ~16 MB | 200 (102 sqlx at 3 projects) |
| 4 | 5 | 760 → 796 MB | ~7 MB | 167 (87 sqlx at 6 projects) |

One run each, so the numbers are rough (idle readings vary by ±100 MB between starts), but
connections per database followed the setting exactly and growth per project roughly halved.

## Decision
- The gateway sets `db_max_readers` from `DB_MAX_READERS` (default **4**) and
  `incoming_request_concurrency_limit` from it as Holochain does (readers − 3, at least 1).
- `/readyz` also reports `threads` and `cpus` (the count Holochain would size its pools by),
  so a session without Railway access can see the effect live.

## Consequences
- Fewer threads and page caches per project; the saving is larger where the container
  reports more CPUs (the live node's CPU count shows in `/readyz` after the deploy).
- Under heavy load, zome calls and authority responses may wait for a reader sooner; raise
  `DB_MAX_READERS` on Railway if `/readyz` or the logs show pool timeouts.
- Still per project: the two databases, their remaining connections, and a kitsune space with
  its gossip state. The next step, if growth stays a problem, is the archivist: does it need
  a full cell per project, or a lighter read path (ADR 0031, 0037)?

## Amended by ADR 0065 (2026-10-28)
The cap belonged to one-network-per-project. In the commons every public space shares one
database, and 4 readers (with 1 concurrent incoming request) left local reads waiting up to
85 s during browser tests, so project chats looked empty. The default is now **16** (Holochain's
2 × CPUs on the 8-CPU seed node). A `DB_MAX_READERS` set on Railway overrides it.
