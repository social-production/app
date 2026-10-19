# Social Production

An open-source coordination platform for collective projects, shared means of production
and liquid democracy. It has no central database, no admin keys and no server it depends on. It runs on
[Holochain](https://www.holochain.org/): it starts from one cloud seed node and grows into a
network of desktops, Raspberry Pis and phones.

- How it fits together: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- Why it is built this way: [docs/decisions/](docs/decisions/)

## Status (2026-10-19)

**M2 (projects) is live** on the test network: https://ui-production-333c.up.railway.app
(it runs alongside the current FastAPI site; that site is unaffected).

What works today:
- **Projects:** start one through the "+" menu (type → basics → location → tags → overview), and
  find it in the feed (Trending / Most recent, Today / This week / This month / All time) and in
  search. Join and leave, support or oppose, open the member list; the creator edits the charter
  (until M3 proposals). New members can start 2 projects a week; full members have no limit.
- Each project is its own network; the seed node's archivist keeps every project available and
  serves the feed (ADR 0030, 0031). Code-only releases no longer reset anything (ADR 0029).
- **The interface is `web/`'s**: app shell, onboarding, feed cards, project page, create flow,
  sheets, profile header with Vouch, settings (profile, devices, passkey, your key, sign out).
- **Identity (M1):** a 24-word recovery phrase *is* the account, optional passkeys, device pairing
  shown as one person, standing with founders (**thebestdamn**, **BigBird**), 2 vouches → Full.

Measured locally: a new project shows in another person's feed within one archivist round (≤15 s);
signup ≈ 7 s.

Known gaps:
- The M2 deploy changed the directory DNA, so the test network was reset again: sign in with your
  24 words and re-enter your profile; founders vouch again.
- Guests can't browse the feed signed out yet; feed cards don't show your own signal; no comments
  until M4 (see docs/plans/M2-projects.md#as-built-2026-10-19).
- iOS phone nodes wait on an upstream Holochain keystore fix (ADR 0020/0021). Android builds.

## Getting started

```sh
# once: install Nix (https://install.determinate.systems), then trust Holochain's
# binary cache — without this, Nix compiles Holochain from source (~1 h):
printf 'extra-trusted-substituters = https://holochain-ci.cachix.org\nextra-trusted-public-keys = holochain-ci.cachix.org-1:5IUSkZc0aoRS53rfkvH9Kid40NpyjwCMCzwRTXy+QN8=\n' \
  | sudo tee -a /etc/nix/nix.custom.conf && sudo systemctl restart nix-daemon

nix develop --accept-flake-config   # holochain, hc, rust+wasm, node
npm install
bash scripts/build-happ.sh  # zomes → DNAs → hApp
cargo test --workspace      # pure rules + scenario tests
bash scripts/dev-seed.sh    # local seed node on :8080 (another terminal)
npm run dev:ui              # VITE_SP_GATEWAYS=http://localhost:8080
bash scripts/e2e-local.sh tests/e2e/accounts.mjs tests/e2e/projects.mjs   # browser tests (seed node from dev-seed with RP_ORIGIN=http://localhost:4174)
```

## Roadmap

Every milestone ends deployed to `social-production-p2p`, verified by the automated tests in
`tests/` (scenario + browser); Grokbot sheets on request.

| | Milestone | Scope | State |
|---|---|---|---|
| **M0** | Foundation | repo, docs, CI, `health` zome in 3 DNAs, gateway, UI shell, desktop app, Railway seed + bootstrap, platform check | ✅ done |
| **M1** | Identity | recovery-phrase accounts, optional passkeys, hosted agents, profiles, device pairing, vouches + standing, founders | ✅ live (M1.1 fixes deployed) |
| **M2** | Projects | `web/`'s design system + shell; project clones, membership, signals, archivist, feed/search (`sp_index`), project screens ([plan](docs/plans/M2-projects.md)) | ✅ live |
| M3 | Governance | Proposal/Vote/Outcome, kinds: edits, phases, updates, links, plans | next |
| M4 | Talk | discussion, project chat, DMs (conversation DNA), in-app notifications | |
| M5 | Work | activities, roles, events, help + service requests, communities/channels | |
| M6 | Platform | moderation + bot reviews, calls, map/places, push relay, software PR/merge flow, board receipts | |
| M7 | Mobile + cutover | Tauri Android/iOS light nodes, multi-seed failover, replace the FastAPI site | |

## License

AGPL-3.0-or-later. Holochain itself is CAL-1.0.
