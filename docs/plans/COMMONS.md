# The commons (ADR 0065): one sharded network for public spaces

Goal: tens of thousands of public projects, events, help requests, communities and channels,
with any full node able to archive all of them. Public spaces become records in the project
DNA's base cell; private spaces stay clones of the same DNA. Each phase ends green and measured.

## Phase 0: measure (before rewriting) — done 2026-10-28 (ADR 0065 "As built")
- `/readyz` has jemalloc figures and app/cell counts (2026-10-28).
- A bench on a local release seed node (`scripts/kept-seed.sh`): create 100, 300 and 1,000
  projects through the zomes (not the browser), recording RSS, threads, open files and CPU.
  Repeat with the commons after Phase 2. Kept in `docs/NETWORK.md`.

## Phase 1: integrity (project DNA, one reset) — built 2026-10-28
- Every entry and link names its space: `space: ActionHash` (the charter's create hash).
  Bases: the charter hash for members, signals, proposals, values, activities, organizers, chat
  and links between spaces; no global anchors.
- `space_of(record)` → the cited charter, read with `must_get_valid_record`: creator (its
  author), kind/mode, audience and governance (ADR 0061 moves from the seed to the charter for
  public spaces; private clones keep the seed check as well).
- In a clone (private space) exactly one charter may exist; in the base cell any number of public
  charters, never private ones.
- Time buckets: `recent.<UTC hour>` path links to the space on each write (coordinator-written,
  validated as "points at a charter").
- Crate rules stay in `sp_governance`/`sp_event`; `sp_core` untouched if possible (it would also
  reset the conversation network).

## Phase 2: coordinators, gateway, index — built 2026-10-28 (cards not yet persisted)
- Coordinators take `space` on every call (the cell is the commons for public spaces).
- Archivist: one cell for the commons; reads `recent.*` buckets since its last round, re-reads
  those spaces only; cards persisted in `gateway.sqlite`, so a restart serves the feed at once.
- Guest reads (`public.rs`) and the people index read the commons cell.
- Directory listings stay, seeded `<creator>.space.<charter hash>` (no directory reset).
- Scenarios: `Net` creates spaces in one shared cell instead of one clone per test project.

## Phase 3: UI — built 2026-10-28 (addressing kept: listing ids and seeds)
- Routes unchanged. `holo/cells.ts` `projectCell(seed)` returns the commons cell tagged with
  the space (`holo/spaces.ts`) and `callCell` wraps calls as `{space, input}`.
- Notifications, the rail and Messages' Public tab read spaces from the commons instead of
  "clones on this node".

## Phase 4: private spaces cold when idle (ADR 0064 step 2)
- Gateways disable a private clone's cells when unused for some hours and enable them on open.

## Later (own ADRs)
- Chats as end-to-end-encrypted messages in the shared conversation network.
- Home gateways for hosted accounts; arc policy switch (ADR 0037) measured on the commons.
