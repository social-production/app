# Architecture

Social Production is a coordination platform with no central database and no master keys.
It runs on Holochain: every user has their own signed source chain, and shared data
lives in DHTs (one per DNA) held by peers. This sheet tells you where everything is and how data moves.
Decisions and their reasons are in [`decisions/`](decisions/).

## The picture

```
 phone PWA / browser             desktop app (Tauri)           phone app (Tauri, M7)
 ───────────────────             ───────────────────           ─────────────────────
 ui/ (static SPA)                ui/ + conductor (IPC)         ui/ + conductor (IPC)
   │ thin client                   │ full node (full arc)        │ light node (small arc)
   │ wss + app token               │                             │
   ▼                               │                             │
 seed node (Railway, + community nodes later)                    │
 ┌──────────────────────────────┐  │                             │
 │ gateway  (services/gateway)  │  │                             │
 │  passkeys, hosted agents,    │  │                             │
 │  app tokens, /ws proxy       │  │                             │
 │  + embedded conductor 0.7    │◄─┴──── gossip (kitsune2/iroh) ─┘
 │  hosted agents' cells        │
 └──────────────────────────────┘
        ▲  peer discovery + relay
        │
 bootstrap service (kitsune2-bootstrap-srv + iroh relay)
```

Every node runs **the same DNAs**. A node's role (seed, desktop, phone) only changes how
much of the DHT it stores and whether it hosts other people's agents. See [NETWORK.md](NETWORK.md).

## DNAs (separate networks)

| DNA | Instances | Holds | Who holds it |
|---|---|---|---|
| `directory` | one, global | profiles, standing + vouches, project index, feeds, places | every full node |
| `project` | a clone per project | membership, proposals/votes/outcomes, discussion, chat, activities, events, help/service requests | its members + seed nodes |
| `conversation` | a clone per DM / group chat | messages | participants only |

All three include the shared `health` zome (ping/heartbeat). Domain zomes arrive per milestone.

## Layers and where code lives

```
Rust (network rules)                               TypeScript (UI)
────────────────────                               ───────────────
crates/sp_*        pure rules, unit-tested         ui/src/routes/        thin pages
   ▲                                                  ▼
zomes/*/integrity  validation = the network's law  ui/src/lib/features/  screens, per feature
dnas/*/zomes/integrity                                ▼
   ▲                                               ui/src/lib/services/  queries + commands
zomes/*/coordinator  zome API (reads/writes)          ▼
dnas/*/zomes/coordinator                           ui/src/lib/holo/      ONLY Holochain client code
   ▲                                                  ▼
services/gateway   seed-node HTTP + admin API      @holochain/client → node
native/            Tauri + Holochain runtime-tauri (conductor in-process, IPC)
```

Arrows point toward dependencies, and nothing may skip a layer. `scripts/check-boundaries.sh`
enforces this. Full rules are in [CODE_RULES.md](CODE_RULES.md).

- **Integrity vs coordinator.** Integrity zomes are the network's law: entry/link types and
  `validate`. Changing them changes the DNA hash, which creates a new network (see DEPLOYMENT.md#versions).
  Coordinator zomes are the API and can be upgraded without forking the network.
- **Pure crates.** Rules such as quorum, tally, standing and limits live in `crates/` with no HDK, so
  validation logic is unit-tested in milliseconds and shared with the gateway and native app.
- **Shared zomes** (used by several DNAs) live in `zomes/<name>/`. DNA-specific zomes live in
  `dnas/<dna>/zomes/{integrity,coordinator}/<name>/`.

## Where does X live?

| I want to… | Go to |
|---|---|
| change what the network accepts | `zomes/*/integrity` or `dnas/*/zomes/integrity`, rule in `crates/sp_*` |
| add a zome function | the coordinator zome, then a service in `ui/src/lib/services/` |
| add a screen | `ui/src/lib/features/<feature>/`, route in `ui/src/routes/` |
| change how the UI finds nodes | `ui/src/lib/holo/nodes.ts` |
| change governance maths | `crates/sp_governance` (+ tests), see [GOVERNANCE.md](GOVERNANCE.md) |
| change seed-node hosting/login | `services/gateway`, see [IDENTITY.md](IDENTITY.md) |
| change Railway setup | `deploy/`, see [DEPLOYMENT.md](DEPLOYMENT.md) |
| test several agents together | `tests/scenarios` (Holochain sweettest) |

## Data flow example (M0 heartbeat)

1. `HealthPanel.svelte` (feature) calls `postHeartbeat()` in `services/health.ts`.
2. The service calls `callZome('directory', 'health', 'post_heartbeat', note)` in `holo/`.
3. `holo/` picks a node (local conductor, else gateways in order), opens an app websocket and calls the zome.
4. The coordinator writes the entry and an anchor link. The author's conductor runs integrity
   `validate`, then publishes. Peers validate again before storing.
5. Another node's `list_heartbeats` sees the link after gossip.

## Status

M0 and M1 are live; see the Status section and roadmap in [../README.md](../README.md).
