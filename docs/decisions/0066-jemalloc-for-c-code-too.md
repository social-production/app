# 0066. jemalloc replaces malloc for the whole process (SQLite included)

Status: **accepted** (2026-10-28). Follows [0060](./0060-jemalloc-on-the-seed-node.md) and
[0064](./0064-restart-after-upgrade.md).

## Context
After the commons (ADR 0065) the live seed node idled at ~795 MB with 3 apps and 9 cells, but
`/readyz` showed jemalloc's resident heap at only 311 MB. `tikv-jemallocator` by default only
serves Rust's `#[global_allocator]`; C code in the same binary (the bundled SQLite and its page
caches, above all) still calls glibc's `malloc`, whose per-thread arenas keep freed memory
(the same waste ADR 0060 removed for Rust allocations). 8 CPUs on Railway mean up to 64 arenas.

`/readyz` gains `maps`: resident memory by kind, from `/proc/self/smaps` (`anonMb`,
`anonExecMb` = wasmer's compiled zomes, `brkHeapMb` = glibc's main arena, `fileMb`,
`stackMb`). `anonMb` minus `heap.residentMb` is what lives outside jemalloc.

Measured locally (release, 4 CPUs, `scripts/cloud-seed.sh`), the same four browser test files
(`private-events`, `projects`, `public-chats`, `events`: 8 apps, 26 cells), idle 100 s after:

| Allocator | RSS | jemalloc resident | anon outside jemalloc | wasmer code | files |
|---|---|---|---|---|---|
| jemalloc for Rust only (ADR 0060) | 788 MB | 413 MB | ~213 MB | 94 MB | 65 MB |
| jemalloc for everything (this ADR) | **601 MB** | 457 MB | ~0 | 94 MB | 65 MB |

## Decision
- The gateway turns on `tikv-jemallocator`'s `unprefixed_malloc_on_supported_platforms`, so
  jemalloc exports `malloc`/`free` and every allocation in the process (SQLite, other C
  libraries) goes through it. Still only in the `jemalloc` feature (Docker and `RELEASE=1`).
- `/readyz` reports `maps`, so the live split can be read without Railway access.

## Consequences
- ~24% less idle memory locally; live (8 CPUs, more arenas) should save more. Check
  `/readyz` after the deploy: `maps.anonMb` should sit close to `heap.residentMb`.
- jemalloc's figures now cover SQLite too, so `heap.allocatedMb` is higher than before for the
  same data; compare RSS across deploys, not `allocatedMb`.
- What remains: wasmer's compiled code (~94 MB, one copy per DNA, ADR 0053), the binary, and
  live data. Cold projects (ADR 0064) stay the next lever as data grows.
