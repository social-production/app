# Deployment

## Railway project `social-production-p2p` (ADR 0014)

Separate from the live FastAPI project `social-production`, so nothing is shared.

| Service | Source | Volume | Public URL used by |
|---|---|---|---|
| `bootstrap` | `deploy/bootstrap/Dockerfile` (`deploy/railway/bootstrap.json`) | none | every conductor (bootstrap + relay) |
| `seed-node` | `deploy/seed-node/Dockerfile` (`deploy/railway/seed-node.json`) | `/data` (conductor DB + lair keystore) | UI (`/session`, `/ws`) |
| `ui` | `ui/build` served statically | none | people |

### seed-node environment

| Var | Meaning |
|---|---|
| `LAIR_PASSPHRASE` | unlocks the keystore. **Losing it loses every hosted agent's key.** Store it in a password manager too |
| `BOOTSTRAP_URL` | `https://<bootstrap service domain>` |
| `RELAY_URL` | defaults to `BOOTSTRAP_URL` (same server) |
| `ADMIN_PORT` | conductor admin port, localhost only (default 4444) |
| `PORT` | set by Railway. The gateway listens here and proxies the app websocket on `/ws` |

Railway exposes a single port per service, so the gateway proxies the conductor's app websocket.
The admin websocket is never exposed.

### Deploy loop

Push `main`, CI runs `scripts/check-all.sh`, Railway rebuilds the services, and then you check
`https://<seed-node>/readyz` = 200. Each milestone ends with a numbered Grokbot retest sheet.

## Versions

- **Holochain** is pinned in three places that move together: `flake.nix` (`holonix main-0.7`),
  `Cargo.toml` (`hdi`/`hdk`/`holochain`), `ui/package.json` (`@holochain/client 0.21.x`), plus
  `HOLOCHAIN_VERSION` in the seed-node Dockerfile.
- **DNA versioning.** Any integrity-zome change makes a new DNA hash, and therefore a new, empty network.
  Before launch that's fine (wipe and redeploy). After launch, integrity changes ship as a
  new hApp version, and data moves by **genesis roll-up**: the new DNA's genesis cites the
  final state of the old network (the design sheet's "schema evolution"). Coordinator-only changes
  hot-swap without a new network. Prefer them.
- **Reproducible DNAs.** A DNA hash is a hash of its WASM, and nodes only meet if their hashes
  match. `scripts/build-zomes.sh` strips machine paths (`--remap-path-prefix`), and Docker
  pins the exact Rust of the Nix shell (`rust:1.95.0`). Verified in M0: identical hashes from
  the Nix build and the seed-node Docker build. Bump both together.
- **Breaking-change rule.** Bump `happ.yaml`'s DNA network seed only on a deliberate network reset,
  never by accident.
