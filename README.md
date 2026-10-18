# Social Production

An open-source coordination platform for collective projects, shared means of production
and liquid democracy. It has no central database, no admin keys and no server it depends on. It runs on
[Holochain](https://www.holochain.org/): it starts from one cloud seed node and grows into a
network of desktops, Raspberry Pis and phones.

- How it fits together: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- Why it is built this way: [docs/decisions/](docs/decisions/)

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
npm run dev:ui
```

## Roadmap

Every milestone ends deployed to `social-production-p2p` with a Grokbot retest sheet.

| | Milestone | Scope |
|---|---|---|
| **M0** | Foundation | repo, docs, CI, `health` zome in 3 DNAs, gateway health, UI shell, Tauri desktop, Railway seed + bootstrap, platform check (Railway, Linux desktop, Android, iOS) |
| M1 | Identity | profiles, passkey login, hosted agents, recovery phrase, vouches + standing |
| M2 | Projects | project clones, membership, project index, feeds/search (`sp_index`) |
| M3 | Governance | Proposal/Vote/Outcome, kinds: edits, phases, updates, links, plans |
| M4 | Talk | discussion, project chat, DMs (conversation DNA), in-app notifications |
| M5 | Work | activities, roles, events, help + service requests, communities/channels |
| M6 | Platform | moderation + bot reviews, calls, map/places, push relay, software PR/merge flow, board receipts |
| M7 | Mobile + cutover | Tauri Android/iOS light nodes, multi-seed failover, replace the FastAPI site |

## License

AGPL-3.0-or-later. Holochain itself is CAL-1.0.
