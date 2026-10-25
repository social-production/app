# 0053. Patch Holochain so compiled zomes are loaded once, not once per new cell

Status: accepted (2026-10-25). Same approach as [0034](./0034-lair-frame-limit.md).

## Context
On 2026-10-25 Railway reported "Deploy Ran Out of Memory" for `seed-node`. Measured locally on
a release build (`RELEASE=1 bash scripts/dev-seed.sh`): every new hosted person cost about
**350 MB** that was never given back. Disabling their app freed nothing, nor did jemalloc, so
it was live memory, not fragmentation. A jemalloc heap profile (`--enable-prof`, `jeprof
--inuse_space`) put it in `ModuleCache::get` → `from_serialized_module`.

The cause is in Holochain 0.7.1 plus wasmer 7.1:
- After each new cell's genesis, `RealRibosome::genesis_complete` evicts that DNA's zomes from
  the conductor's shared in-memory module cache, so the next call reloads them from the
  database. The module cache also drops entries idle for an hour.
- Every reload deserializes the module into wasmer's shared runtime engine, which pushes its
  machine code onto `EngineInner::code_memory` and never removes it. The engine lives as long
  as the conductor.

So each sign-up (three cells), each chat (a clone), each project someone opened, and every
hour a zome sat unused added another copy of the compiled zomes.

## Decision
- Vendor `holochain` 0.7.1 (`vendor/holochain`, `[patch.crates-io]`) with two changes:
  `genesis_complete` no longer evicts, and the module cache has no time-to-idle. A module stays
  loaded once per wasm hash. There are a few dozen at most, and they were loaded anyway.
- Considered and dropped: putting idle hosted apps to sleep (disabling them). It was built
  and measured, and it freed nothing, because the leaked code isn't owned by the app. It would
  also make sleeping people unreachable for calls and stop them serving their chats.
- Report upstream (Holochain: the genesis eviction; wasmer: engine code memory is never freed),
  and drop the patch once a release fixes either.

## Consequences
- Measured: 3 new accounts +110 MB (the first load of the shared modules), the next 3 +8 MB.
  Seven browser test files (~14 accounts, projects, chats, calls, files) took the node from
  1.1 GB to 1.4 GB. Before, 3 accounts alone added 0.9–1.2 GB.
- The vendored crate must move with Holochain upgrades (CLAUDE.md: versions move together).
  Re-check the leak after any upgrade with the measurement above.
- The native app and the scenario tests use the patched conductor too.
