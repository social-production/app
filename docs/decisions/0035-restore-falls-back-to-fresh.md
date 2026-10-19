# 0035. A phrase restore waits for the chain, then starts fresh if there is none

Status: accepted (2026-10-19)

## Context
Signing in with a phrase this seed node has never hosted installs the app with
`restore_from_dht`, so the person's existing chain is rebuilt from the network instead of
starting a second one (a fork). The gateway enabled the app straight after installing, but in
Holochain 0.7 a restore runs in the background and the app can't be enabled until it finishes. For a
phrase with no chain on this network (never used, or used only before a test-network reset), the
restore never finishes, and sign-in hung.

## Decision
After a restore install, the gateway waits up to 45 s for the restore to settle, then enables the app
(`services/gateway/src/node/restore.rs`). If nothing settles, or the restore fails for good, it
uninstalls and installs a fresh chain.

## Consequences
- Signing in with any valid phrase works. Before a reset, people get their chain back; on a new
  network they start fresh.
- The seed node holds the whole directory network, so a chain that exists arrives well within 45 s.
  If a chain lives only on nodes that are offline at that moment, the person gets a fresh chain
  that conflicts with the old one. Native nodes restore through their own conductor, without this fallback.
