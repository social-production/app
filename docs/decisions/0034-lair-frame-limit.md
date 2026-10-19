# 0034. Keep keystore entries few, and patch lair's 8 KiB message limit

Status: accepted (2026-10-19)

## Context
After the M2 deploy, signup on the seed node failed with `ConnectionAborted`. Before installing an
app for a given agent key, the conductor (holochain 0.7.1 `install_app_bundle`) calls
`list_public_keys`, which asks lair for **every** entry in one message. Lair (0.7.1) caps a message
at 8 KiB (`MAX_FRAME` in `sodium_secretstream.rs`), which is about 55 entries. Lair can't delete entries.
The seed node had grown past that, because every signup also created a one-off import key and every
start a warm-up key. `tests/lair_capacity.rs` reproduces it.

## Decision
- **Few entries:** one key per hosted person (short tag `p-<24 hex>`), one shared import
  recipient (`n-import`), and one archivist key for the life of the node (`n-archivist`). Warm-up
  calls the archivist's cells instead of installing a throwaway app with a fresh key.
- **Patch lair** (vendored at `vendor/lair_keystore_api`, `[patch.crates-io]`): `MAX_FRAME` goes to
  64 KiB, the most its u16 length prefix allows, so a node holds roughly 450 people. The client and
  server run in the same binary, so both sides agree. Report upstream and drop the patch once
  Holochain stops listing every key, or lair raises the limit.

## Consequences
- The live seed node recovers without a reset. Its ~80 junk entries stay, at the cost of a little capacity.
- One seed node hosts on the order of 450 browser people. Beyond that, more seed nodes (M7) or
  native nodes, which hold only their own keys.
- The native app uses the same patched lair; it has a handful of keys, so it makes no difference there.
