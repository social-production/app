# Vendored crates

Patched copies of upstream crates, applied workspace-wide with `[patch.crates-io]` in the root
`Cargo.toml`. Each patch is as small as possible and has an ADR. Drop the copy once upstream fixes it.

| Crate | Upstream | Patch | Why |
|---|---|---|---|
| `lair_keystore_api` 0.7.1 | crates.io (MIT OR Apache-2.0) | `MAX_FRAME` 8 KiB → 64 KiB in `src/sodium_secretstream.rs` | The conductor lists every key in one lair message on each app install; at 8 KiB a seed node stops installing after ~55 people (ADR 0034). |
