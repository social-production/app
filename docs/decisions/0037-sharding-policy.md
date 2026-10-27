# 0037. Sharding policy: full copies while small, sharded by default later, archivers by choice

Status: accepted (2026-10-19). Extends [0036](./0036-every-full-node-keeps-the-network.md).

## Decisions (owner, 2026-10-19)
- **Switch point: automatic by node count.** Full nodes (desktops, Pis, servers) hold everything
  until there are about **30 always-on full nodes**. After that, Holochain's automatic arc sizing
  becomes the default and each node holds a slice. The node decides from the peers it sees, so no
  release has to flip a switch.
- **Redundancy: 20+ copies** of every piece of data once sharded. Arcs are sized for that, which
  favours safety over disk use (volunteer laptops sleep, Pis lose power).
- **Archivers: opt-in for anyone.** Any full node can tick "archive everything": the whole
  directory, every project, and the feed index (ADR 0031). Seed nodes are archivers with a
  gateway. Sharding never turns this off.

## Phones (decided 2026-10-19)
- Phones are **light**: they hold no one else's data and use a node (their own desktop or Pi, a
  community node, or a gateway), which is what app stores and batteries need. Search and the feed
  come from that node, so a light phone still finds everything.
- Later, **Android only**: an opt-in "help the network while charging on Wi-Fi" that holds a small
  slice. iPhone gets no such mode, because iOS can't run it in the background.

## Desktop and Pi: the person chooses (decided 2026-10-19)
Every full node gets a **Storage** setting, with a default that depends on the product:

| Mode | What it holds | Default for |
|---|---|---|
| Light | Only its own chain; asks other nodes for the rest | — |
| Share a slice | Its own chain plus a slice of every network, sized automatically | desktop app |
| Archive everything | Every network and every project, plus the feed index | Pi image, seed and community nodes |

- **Storage limit:** a "use at most N GB" field. The node keeps its slice inside that budget by
  shrinking its arc (Holochain's storage arc), never by dropping its own chain or projects it
  joined. Archive mode warns when the limit is too small to hold everything.
- **Compression and a hot cache (to investigate before building):**
  - Holochain keeps data in SQLite and has no per-record compression.
  - Options, in order: (1) compress large fields ourselves (avatars and long text are already
    small, files arrive in M5), (2) recommend filesystem compression (zstd on btrfs/ZFS) on the
    Pi image, (3) Holochain's own fetch cache already keeps recently read data close, so hot and
    trending items come back fast. We measure it, then decide whether a separate hot cache helps.

## How (implementation notes)
- `sp_core::network` gains the arc policy: full arc below the threshold, Holochain's target arc
  factor for 20+ copies above it. Both the seed node and the desktop app read it.
- "Archive everything" becomes a setting of the shared node crate (ADR 0036). It keeps full arc in
  every network, joins every project clone, and builds the feed index.
- The Storage setting lives in the native app's settings (and the Pi image's config file); light mode
  sets the arc to zero.
- Measured, not assumed: the docs/NETWORK.md timings get a sharded run before the default flips.

## Update (owner, 2026-10-27)
A person can always choose to be a full node (whole network plus the feed index), whatever the
default becomes; sharding comes later.
