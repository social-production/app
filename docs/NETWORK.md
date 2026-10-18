# Network

How the platform keeps running when any one machine, including Railway, disappears.

## Node types

| Node | Runs | Stores | Hosts others' agents | Always on |
|---|---|---|---|---|
| **seed** (Railway, later VPS/Pi) | conductor + gateway | everything (full arc) | yes (PWA users) | yes |
| **full** (desktop app, Pi) | conductor in Tauri | everything until sharding (full arc) | no | often |
| **light** (phone app, M7) | conductor in Tauri | own chain + member projects + small arc | no | only while open |
| **thin** (PWA/browser) | nothing | nothing | — | — |

- A thin client is a UI connected to someone else's conductor. It has no keys of its own
  in the browser; its agent is hosted (see IDENTITY.md).
- Phones become light nodes in M7. The zomes are written so the same code runs unchanged at
  any arc size. iOS needs Holochain's `wasmer-wasmi` interpreter build because Apple forbids
  the WASM JIT; the M0 platform check verifies this.
- Mobile OSes suspend apps in the background. Phones sync **through** always-on peers. They
  never rely on meeting another phone directly.

## Discovery and transport

- **Bootstrap** (`kitsune2-bootstrap-srv`) tells nodes where peers are for each DNA.
- **Relay** (iroh relay, built into the same server) carries traffic between nodes that can't
  connect directly (NAT, mobile networks).
- Both run as the Railway `bootstrap` service. Clients ship a **list** of bootstrap/relay URLs,
  so community-run copies make this service replaceable too.
- Railway exposes HTTP(S) only, with no public UDP, so peers reach the seed node through the relay.
  Direct QUIC is used between peers that allow it.

## Failover (ADR 0013)

`ui/src/lib/holo/nodes.ts` builds the endpoint list:

1. Inside the native app: the device's own conductor over Tauri IPC (ADR 0021).
2. Gateways from `VITE_SP_GATEWAYS`, in order (Railway first, then community nodes).
3. M1+: an updatable list fetched from the directory DNA (community nodes announce themselves).

If a node drops, the next zome call reconnects to the next endpoint. If every hosted node is gone,
hosted users restore their recovery phrase into a desktop/phone app and carry on (IDENTITY.md).

## Observed timings (M0, local)

- Local network: a freshly started node received a heartbeat after ~60 s (first gossip
  round); an already-connected node had it within 0.1 s.
- Over the internet via the Railway relay: two nodes that had already met synced in 2.5 s.
  **First contact between the Railway seed node and a new home node took more than 4 minutes**
  (gossip rounds timed out over the relay before converging). All nodes did converge.
  This needs tuning before real users (M1): gossip initiate interval, timeouts, and whether
  Railway can expose the relay's QUIC/UDP port for direct paths.

## Storage growth and sharding

- Until the network is large, every full node uses a **full arc** (stores everything).
- Project and conversation data is already split by DNA clone, so it is naturally held by
  members only. This is the first level of sharding and costs nothing.
- Later, full nodes lower `target_arc_factor` when the directory DHT outgrows a desktop.
  This is a config change; the code stays the same.

## Risks being watched

- **Hosted-agent scale.** Each PWA user has a cell per DNA clone they use, all on the seed
  conductor. Plan: several seed nodes split hosted users (gateway routes by agent).
- **Relay dependence.** A relay can be a choke point, so we keep several and allow community-run ones.
