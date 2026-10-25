# Vendored crates

Patched copies of upstream crates, applied workspace-wide with `[patch.crates-io]` in the root
`Cargo.toml`. Each patch is as small as possible and has an ADR. Drop the copy once upstream fixes it.

| Crate | Upstream | Patch | Why |
|---|---|---|---|
| `lair_keystore_api` 0.7.1 | crates.io (MIT OR Apache-2.0) | `MAX_FRAME` 8 KiB → 64 KiB in `src/sodium_secretstream.rs` | The conductor lists every key in one lair message on each app install; at 8 KiB a seed node stops installing after ~55 people (ADR 0034). |
| `holochain` 0.7.1 | crates.io (CAL-1.0) | `genesis_complete` no longer evicts modules (`src/core/ribosome/real_ribosome.rs`); the module cache has no time-to-idle (`real_ribosome/module_cache.rs`) | A reloaded module's machine code is added to wasmer's shared engine, which never frees it, so each new cell leaked ~100 MB; the seed node ran out of memory (ADR 0053). |
