# app — Social Production P2P (Holochain 0.7)

The P2P rebuild of Social Production. It runs in parallel with the live FastAPI site
(`../web`, `../web-backend`) until parity, then replaces it. Start with
`docs/ARCHITECTURE.md`. The reasons behind every choice are in `docs/decisions/`.

## Layout
- `crates/sp_*`: pure Rust rules (governance maths, limits, index). No HDK. Unit-test here first.
- `zomes/<name>/{integrity,coordinator}`: zomes shared by several DNAs (`health`).
- `dnas/{directory,project,conversation}/`: `dna.yaml`. DNA-specific zomes go in `dnas/<dna>/zomes/{integrity,coordinator}/<name>/` (add the glob to root `Cargo.toml` when the first one lands).
- `happ/`: `happ.yaml` (roles; project/conversation are cloned) + `web-happ.yaml`.
- `services/gateway`: the seed node, one process (ADR 0023): embedded conductor + axum API (`/account/*` signup/restore/login with passkeys + proof-of-work, `/session/resume`, `/ws` proxy). Its own `gateway.sqlite` maps passkeys and device keys to hosted agents.
- `ui/`: SvelteKit 3 static SPA. Layers: `routes → lib/features → lib/services → lib/holo`. Only `lib/holo` imports `@holochain/client`.
- `native/`: Tauri 2 + Holochain's `tauri-plugin-hc` (ADR 0021). The UI talks to the device's conductor over IPC; browsers use gateways.
- `deploy/`: seed-node + bootstrap Dockerfiles, Railway configs. `tests/scenarios`: sweettest multi-agent tests.

## Rules
- Layer and size rules are CI-enforced: `docs/CODE_RULES.md`. Files ≤300 lines, functions ≤60.
- UI imports use `#lib/...` **with extensions** (`#lib/holo/index.ts`). `$lib` no longer exists in Kit 3.
- An integrity zome change means a new DNA hash, which means a new network. Prefer coordinator-only changes. Size limits live in `sp_core::limits`.
- The three DNAs differ only by `properties.kind` until their own zomes land. Don't remove it, or they collapse into one network.
- Own data reads (`get_profile`/standing for yourself) come from your own chain (`records::own_latest`), because network reads lag right after a write.
- Holochain versions move together: `flake.nix`, `Cargo.toml` (`hdi =0.8.1`, `hdk =0.7.1`, `holochain =0.7.1`), `@holochain/client 0.21.x`, the Dockerfile's `HOLOCHAIN_VERSION`.
- New decisions get an ADR (`docs/decisions/NNNN-*.md`, plus the index in its README).
- The workspace path contains a space ("Social Production"). `flake.nix`'s shellHook strips stdenv's `-rpath $out/lib` and sets `OPENSSL_NO_VENDOR=1` because of it. Quote paths in scripts.
- Commit messages: one plain sentence describing the user-visible effect.

## Checks (inside `nix develop --accept-flake-config`)
- Everything CI runs: `bash scripts/check-all.sh`.
- Fast loop: `cargo test -p sp_governance`, `npm run check -w ui`, `bash scripts/check-boundaries.sh`.
- Pack: `bash scripts/build-happ.sh` (zomes → 3 DNAs → hApp). Scenarios: `npm run test:scenarios`.
- Local seed node (bootstrap + conductor + gateway on :8080): `bash scripts/dev-seed.sh`.
- UI dev: `npm run dev:ui` (set `VITE_SP_GATEWAYS`, see `ui/.env.example`).

## Deploy
Railway project `social-production-p2p` (services `bootstrap`, `seed-node` with a `/data` volume, `ui`): `docs/DEPLOYMENT.md`.
Never lose `LAIR_PASSPHRASE`. Finish each milestone with a numbered Grokbot retest sheet.

## Status
M0 (foundation) is in progress. See `README.md#roadmap`.
