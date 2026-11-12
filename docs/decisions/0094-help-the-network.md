# 0094. "Help the network": a desktop runs a bootstrap server and announces it (SEEDLESS step 3b)

Status: **accepted** (2026-11-12, cloud session; the owner steers less until parity, STATUS
2026-10-25). Plan: [`docs/plans/SEEDLESS.md`](../plans/SEEDLESS.md) step 3b. Builds on ADR 0091.

## Context
Every node finds the others through a bootstrap server, and reaches them through a relay
(ADR 0091: the saved one, Railway's, then Holochain's public development servers). If
Railway's bootstrap is gone, new installs only have Holochain's development servers, which
are not meant for production and may disappear too. The owner wants desktops to keep the
network going without the seed node (STATUS 2026-11-10), and SEEDLESS asks for desktops that
can be reached from outside to serve as bootstrap and relay, announced in the directory.

## Decision
- **Settings → Network → Help the network** (desktop only): a switch, the port on this
  computer (8787 suggested) and the address other people use (`http(s)://host[:port]`; not
  localhost). Saved in `help-network.json` in the app's data folder; saving restarts the app.
  The person forwards the port (or runs the app on a machine with its own address); the app
  cannot tell from inside whether it is reachable, so Settings says so plainly.
- **At start** (`native/src/helping.rs`), when it is on, the app runs Kitsune's own bootstrap
  server, the same `kitsune2_bootstrap_srv` 0.5.2 Railway runs, in-process, on every IPv4
  address at that port. Its iroh relay is on too (same port, plain HTTP unless the address is
  https behind a proxy); iroh's traffic through a relay is end-to-end encrypted, and
  `network.rs` already lets a node use an `http://` relay. QUIC address discovery is off (it
  needs a certificate).
- **Announcing** is a link in the directory, not a new entry type: the health zome's
  coordinator gets `announce_server(note)` (a link from the `bootstrap_servers` path to the
  author, the note `{"bootstrap":…,"relay":…}` in its tag, at most 280 bytes) and
  `list_servers()` (announcements of the last 14 days, newest first). The integrity zome
  already accepts any heartbeat link, so **no DNA hash changes and no network resets**.
  A helping desktop re-announces each time it starts and connects (`holo/servers.ts`
  `shareServers`, run from `attachNative`).
- **Learning**: once connected, every desktop reads `list_servers` and hands the notes to the
  native `remember_servers`, which keeps up to 8 valid ones (`native/src/announced.rs`, same
  address checks) in `announced-servers.json`. **The next start** tries, in order: the saved
  server, Railway's, this desktop's own server (when helping: its bootstrap at
  `http://127.0.0.1:<port>`, its relay by the public address, since peers reach a node through
  the relay its peer record names), the announced ones, then Holochain's public ones; the first that answers `/health` is used.
  So while Railway's answers nothing changes for anyone; when it doesn't, helpers register
  with their own servers and other desktops fall through to the announced ones.

## Consequences
- Coordinator, native and UI only. Browsers are unaffected (they use a gateway).
- **Anyone can announce.** A note is only used when the servers before it don't answer, and
  every address is checked. A bad bootstrap can hide peers from an install that uses it, but
  cannot forge them (peer records are signed by their agents); a bad relay can drop traffic
  but not read it. A later version may list only announcements by Full members.
- The list is learned at connection and applied at the next start: a desktop that has never
  connected (a new install) only has the shipped list. New installs need either a shipped
  community server (docs/RUN-A-BOOTSTRAP.md) or one connection while something answers.
- A node keeps the bootstrap and relay it started with until it restarts. If Railway's relay
  stops while the app runs, its relayed connections drop until a restart picks another.
- The announcement links grow with every start of every helper (no deletes: heartbeat links
  are permanent). Fine at this size; time-bucket the path if it grows.
- A helper uses its own relay by its public address, so its router must let it reach its own
  forwarded port from inside (most do). An `http://` relay is untested across real routers:
  if iroh peers won't dial it, helpers need https (Caddy in front, as RUN-A-BOOTSTRAP.md).
- Not tested end to end yet: two desktops finding each other through a third desktop's server
  with Railway's bootstrap off (SEEDLESS's test for this step) needs three machines with
  displays, or a CI job that starts several desktop apps.
