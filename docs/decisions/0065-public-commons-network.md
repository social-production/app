# 0065. Public spaces share one sharded network (the commons); clones only for private ones

Status: **accepted** (2026-10-28, owner: "major changes can still be made… we could have tens of
thousands of projects… you have control"). **Supersedes** [0005](./0005-dna-topology.md)'s
"project = cloned per project" and [0030](./0030-project-clones-and-creator-seed.md) for public
spaces. Plan: [`docs/plans/COMMONS.md`](../plans/COMMONS.md).

## Context
Today every project, event, chat (and, as planned, every community, channel and help request) is
its own network: a clone of a DNA with its own hash. Anything that keeps a space available joins
one network per space: the seed node's archivist (ADR 0031), every desktop or Pi in "archive
everything" (ADR 0036/0037), and every gateway-hosted member who opened it.

Measured this week (ADR 0062, 0064), each network on a node costs two SQLite databases with
their own connection pools and threads, a kitsune gossip space with its own rounds and bootstrap
registrations, and a cell with its own workflow loops: ~3.5 MB idle and 7–16 MB busy per project
on a release seed node, plus threads and file handles. At **10,000 projects** an archiver would
hold 20,000 databases, 10,000 gossip spaces and tens of GB of RAM: no seed node or volunteer Pi
survives it. Worse, it defeats Holochain's own sharding: each project is a tiny network every
holder keeps whole, so storage arcs (ADR 0037) can never spread the load. ADR 0005's "sharding by
membership comes for free" turns out to shard the wrong way.

## Decision
1. **The commons:** every *public* space (projects, events, help requests, and later communities
   and channels) is a record in **one network**, the project DNA's **base cell** (no network seed,
   which every person already has). A space's id is its founding charter's create action hash.
   - Every record names its space (`space: ActionHash`); links hang off per-space bases (the
     charter hash), never off global anchors.
   - Validators read the creator, kind, audience and rules from the cited charter
     (`must_get_valid_record`), not from the network seed. A space costs database rows, not a
     network, and Holochain's arcs shard the commons once there are enough full nodes (ADR 0037).
2. **Private spaces keep their own networks**, using the same DNA code: a private event (ADR 0061)
   and later a private community is a clone with a secret seed that holds exactly one space. The
   integrity rules are the same in both modes (the clone's charter is the only one allowed).
   Gateways keep private clones cold when idle (ADR 0064 step 2).
3. **The index follows changes, not polling:** each write also links its space under a time
   bucket (`recent.<UTC hour>`). The archivist re-reads only spaces with new activity and keeps the
   cards in `gateway.sqlite`; every 15 s polling of every project (ADR 0031) goes away. Feeds and
   search stay a local index on every full node (ADR 0012).
4. **Directory listings retire for public spaces:** the commons record is the listing. The
   directory keeps identity, standing, follows and feedback.
5. **Later, in their own ADRs:** direct and group chats become end-to-end-encrypted messages in
   the shared conversation network (keys delivered through the existing encrypted inbox, ADR
   0043), so chats stop costing a network each; and hosted accounts get a home gateway, so
   hosting spreads across seed nodes (any gateway can already restore an account by phrase).

## As built (2026-10-28)
- **Listings stay** (changes point 4): a public space keeps its directory listing, whose seed
  `<creator>.space.<charter hash>` names the space instead of a network. The directory's rules
  (creator in the seed) are unchanged, so the directory network is not reset, and every page,
  link and route that knows a project by its listing id or seed keeps working.
- **Calls name their space:** every project-DNA call about a space takes `{space, input}`. A
  bare input means "this clone's one space", so private events (and anything written before)
  call exactly as before. Founding (`create_charter`) and file parts are not about one space.
  The project zome resolves a clone's space (`space`) and marks activity (`touch`) for the
  chat zome, which can't read the project rules itself.
- **Chat** messages and read marks name their space; a message is listed only under its own.
- **The person's own spaces** (`my_spaces`: their live memberships) replace "clones on this
  node" for the rail, notifications and the Public tab.
- **The archivist** reads every listed space in its one commons cell; it re-reads a space
  when it is new, written to in this or the last hour (`active_spaces`, once a minute), or every
  15 minutes; other cards are kept. Guest reads go to the commons cell the same way.
- **Measured (clones, before):** one conductor creating project clones went from 52 threads and
  100 open files at 1 project to 1,177/2,537 at 100 and 2,522/5,427 at 200 (about 12 threads and
  27 files per project; RSS 1.09 → 1.38 GB). With the commons a project adds rows, not threads.

## Consequences
- One reset of the project network (and the directory, for listings); test data only.
- The rewrite touches the project zomes (~5,400 lines), the archivist and index, the scenarios
  and the UI's project addressing (`cellOf`/`projectCell`, 14 files): about 1–2 weeks, before
  M5d would add communities and channels as more clones.
- Cost per public space on any node: rows in one database. A node's cost grows with the data in
  its arc, not with the number of spaces; archivers stay possible at tens of thousands of spaces.
- One network means one integrity version for all public spaces (as today, since they share a
  DNA hash). Hot global anchors are forbidden by design; per-space bases and time buckets only.
- Private spaces still cost a network each, so they are kept cold when idle; chats move into the
  shared conversation network later.
- Measured before and after (plan Phase 0): RSS, threads, file handles and CPU on a seed node
  with 100/300/1,000 spaces, clones vs the commons.
