# 0029. Hosted apps are versioned by integrity; coordinators are swapped in place

Status: accepted (2026-10-19). Amends [0028](./0028-happ-versions-and-network-resets.md).

## Context
ADR 0028 versioned hosted apps by the sha256 of the whole hApp bundle. Any change, even a
coordinator-only fix, then gave every person a new app id: the old app was removed and signing in
installed a new one. When the DNA hashes are unchanged, that new app is the *same cell* (same DNA,
same agent), so it would run genesis again on top of a chain the network already holds. That is a
fork, not an upgrade.

## Decision
- The version in `person-<agent>-<version>` and `archivist-<version>` is the first 8 hex of the sha256
  of the hApp's sorted base DNA hashes (`services/gateway/src/node/bundle.rs`). It changes only when
  integrity changes, which is exactly when a new network starts.
- At start the gateway reads each role's coordinator zomes from the bundle and calls
  `UpdateCoordinators` for every DNA its running cells use (`node/release.rs`). Clones are included.
  Coordinator releases keep everyone's chain, session and data.

## Consequences
- Coordinator-only deploys are invisible to people: no sign-in, no reset.
- Integrity changes still reset the test network as in 0028, until genesis roll-up is in place for launch.
- The coordinator update runs on every start. It costs little when nothing changed, because WASM is cached by hash.
