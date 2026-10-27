# 0036. Every full node keeps the network, not just the seed node

Status: accepted (2026-10-19). Builds on [0011](./0011-phones-as-nodes.md), [0012](./0012-local-index.md), [0031](./0031-archivist-and-node-index.md).

## Context
The cloud seed node may be shut down, run out of money, or be blocked. Today it holds the whole
directory (full arc), archives every project (ADR 0031) and serves the feed index. People on
desktop apps hold their own data, but only part of the directory, and only the projects they
joined or opened. If the seed node vanished, directory data would survive only if peers' arcs
happened to cover it, and projects with every member offline would be unreachable.

## Decision
- **Full arc on every full node while the network is small:** desktop apps (and Pis) set
  `target_arc_factor: 1` like the seed node, so each one holds the whole directory. Light clients
  (browsers, and phones in light mode) hold nothing and use a node.
- **Archive and index everywhere, not only on the seed:** the archivist (join every listed project)
  and the feed index (`sp_index`) move out of the gateway into a crate any full node runs. The
  desktop app archives every project by default while the network is small, and serves its feed
  from its own index instead of the seed node's. Planned with M7's multi-seed failover;
  tracked in the roadmap.
- **Browser people** are hosted by a gateway. When one goes away they sign in on another gateway
  with their words (ADR 0013), and their chain comes back from whichever nodes hold it.

## Consequences
- Losing the cloud node is an outage for browser people until another gateway exists, but no data
  is lost, as long as at least one full node with the data is online.
- Full arc costs disk and bandwidth that grow with the network. Arcs shrink to Holochain's
  automatic sizing once there are enough full nodes; this is set per node type.

## Update (owner, 2026-10-27)
- Desktop users choose whether their app is a full node, and "full" stays available to anyone
  who wants it, also after sharding arrives (ADR 0037).
- Every full node holds the feed index and runs the archivist, so any one desktop online is a
  complete backup. Until that move (M7) only the seed node's `/data` volume is a full copy.
- No hurry to shard: it waits until there are thousands of users and the space to shard.
