# Social Production

An open-source coordination platform for collective projects, shared means of production
and liquid democracy. It has no central database, no admin keys and no server it depends on. It runs on
[Holochain](https://www.holochain.org/): it starts from one cloud seed node and grows into a
network of desktops, Raspberry Pis and phones.

- How it fits together: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- Why it is built this way: [docs/decisions/](docs/decisions/)

## Status (2026-10-18)

**M1 (identity) is live** on the test network: https://ui-production-333c.up.railway.app
(it runs alongside the current FastAPI site; that site is unaffected).

What works today:
- Sign up with a 24-word recovery phrase (it *is* the identity), sign in with it on any device,
  optionally save a passkey on devices that can store one. There are no passwords or emails.
- **The interface is `web/`'s** (M2a): same app shell, top bar, left rail, phone bottom nav, onboarding,
  profile header with the Vouch button, and settings (profile, devices, passkey, your key, sign out).
- Profiles (name, picture, home area, bio), the People list, and device pairing (shown as one person).
- Standing: founders (**thebestdamn**, **BigBird**) start Full; 2 vouches or 30 days of steady
  activity make someone Full; 5 vouches per 30 days; every node validates the rules.
- Seed node on Railway (one process: gateway + embedded conductor), bootstrap/relay, and a desktop
  app on Holochain's official runtime-tauri. Desktop nodes sync with the seed node over the internet.

Measured on the live site: signup ≈ 9 s, a new person visible to others ≈ 3 s,
first-signup-after-deploy delay removed (zomes are compiled at startup).

Known gaps:
- Projects aren't here yet (M2b/M2c). Until then Home lists people, and the shell's other links
  (notifications, messages, platform, search) are placeholders naming the milestone that fills them.
- iOS phone nodes wait on an upstream Holochain keystore fix (ADR 0020/0021). Android builds.
- Vouching has scenario tests but no browser test yet; bot reviews and vouch suspension come in M6.

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
node tests/e2e/accounts.mjs http://localhost:5173   # browser test (see the file header)
```

## Roadmap

Every milestone ends deployed to `social-production-p2p`, verified by the automated tests in
`tests/` (scenario + browser); Grokbot sheets on request.

| | Milestone | Scope | State |
|---|---|---|---|
| **M0** | Foundation | repo, docs, CI, `health` zome in 3 DNAs, gateway, UI shell, desktop app, Railway seed + bootstrap, platform check | ✅ done |
| **M1** | Identity | recovery-phrase accounts, optional passkeys, hosted agents, profiles, device pairing, vouches + standing, founders | ✅ live (M1.1 fixes deployed) |
| **M2** | Projects | ✅ M2a: `web/`'s design system + app shell, M1 screens redone with it. Next: M2b project clones, membership, signals, archivist, feeds/search (`sp_index`); M2c project screens from `web/` ([plan](docs/plans/M2-projects.md)) | in progress |
| M3 | Governance | Proposal/Vote/Outcome, kinds: edits, phases, updates, links, plans | |
| M4 | Talk | discussion, project chat, DMs (conversation DNA), in-app notifications | |
| M5 | Work | activities, roles, events, help + service requests, communities/channels | |
| M6 | Platform | moderation + bot reviews, calls, map/places, push relay, software PR/merge flow, board receipts | |
| M7 | Mobile + cutover | Tauri Android/iOS light nodes, multi-seed failover, replace the FastAPI site | |

## License

AGPL-3.0-or-later. Holochain itself is CAL-1.0.
