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
- Local seed node (bootstrap + conductor + gateway on :8080, dev founders from `tests/e2e/fixtures/dev-founders.json`): `RP_ORIGIN=http://localhost:4174 bash scripts/dev-seed.sh`.
- Browser tests against it: `bash scripts/e2e-local.sh tests/e2e/{accounts,projects,chat,governance}.mjs`.
- Ported styles: add to `ui/src/lib/components/styles/PORTS`, run `python3 scripts/port-styles.py` (never hand-edit the CSS).
- UI dev: `npm run dev:ui` (set `VITE_SP_GATEWAYS`, see `ui/.env.example`).

## Deploy
Railway project `social-production-p2p` (services `bootstrap`, `seed-node` with a `/data` volume, `ui`): `docs/DEPLOYMENT.md`.
Never lose `LAIR_PASSPHRASE`. Grokbot is paused until ~2026-10-25: Claude tests (scenarios, `scripts/e2e-local.sh`, live smoke); write Grokbot sheets only on request.

## Cloud sessions (claude.ai/code, a fresh clone of this repo)
The owner's machine has the workspace (`../web`, `../web-backend`, Railway login, agent worktrees); a cloud session has only this repo. So:
- **Get web/ for porting:** `git clone --depth 1 https://github.com/social-production/web ../web` (public). `scripts/port-styles.py` and the PORTS file expect it as a sibling of `app/`.
- **Toolchain:** Nix may already be installed; if the proxy blocks GitHub tarballs, run commands through `bash scripts/cloud-nix.sh <cmd>` (fetches flake inputs over git). Otherwise try Nix (`curl -fsSL https://install.determinate.systems/nix | sh -s -- install linux --init none --no-confirm`, add the Holochain cachix lines from README, then `nix develop --accept-flake-config`). If Nix won't install, you can still run `npm ci && npm run check -w ui && npm run lint -w ui`, `bash scripts/check-boundaries.sh`, `bash scripts/check-sizes.sh` and `cargo test -p sp_core -p sp_governance -p sp_index` with a stock Rust toolchain; zomes, scenarios and the seed node need Nix. Say in STATUS.md which checks you could not run.
- **Don't push straight to `main`**: it deploys at once, before CI. Work on a `claude/<topic>` branch and open a PR; CI runs on PRs (~2 h with scenarios, `gh pr checks <n>`). When CI is green, a cloud session may merge it (`gh pr merge <n> --merge`; the owner approved this on 2026-10-19). Merging deploys, so smoke the live site afterwards and update README Status + STATUS.md. If a deploy breaks the site, revert it in a new PR.
- No Railway access: you cannot deploy, read seed-node logs or touch `LAIR_PASSPHRASE`. Read-only live checks (`curl` the URLs in STATUS.md) are fine.
- Before finishing, update `STATUS.md` ("In flight" + what you checked) on your branch so the next session knows.

## Status
**Start with `STATUS.md`**: what's live, work in flight, next steps and hard-won lessons. Update it as you go and at the end of every session; keep `README.md` Status + roadmap current after every deploy.
