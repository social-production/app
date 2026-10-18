# Decision log

One file per decision. To change a decision, write a new ADR that supersedes the old one.

- [0001](./0001-parallel-run-fresh-network.md) Run in parallel with FastAPI; fresh network, no import
- [0002](./0002-one-monorepo.md) One monorepo: social-production/app
- [0003](./0003-monorepo-layout.md) Repo layout
- [0004](./0004-fresh-ui.md) Fresh SvelteKit UI carrying the current design
- [0005](./0005-dna-topology.md) Three DNAs: directory, project (clone per project), conversation (clone per chat)
- [0006](./0006-open-membrane.md) Open, platform-wide membrane; no per-project join rules
- [0007](./0007-cited-standing.md) Standing via cited evidence
- [0008](./0008-vote-eligibility.md) Vote eligibility: member + full standing before the proposal
- [0009](./0009-generic-governance.md) Generic Proposal → Vote → Outcome, with special kinds allowed
- [0010](./0010-hosted-agents-passkeys.md) Hosted agents for PWA users, passkey login, recovery phrase
- [0011](./0011-phones-as-nodes.md) Phones become light nodes
- [0012](./0012-local-index.md) Feeds and search are a local index on every full node
- [0013](./0013-node-failover.md) Clients ship a list of nodes and fail over
- [0014](./0014-railway-p2p-project.md) Separate Railway project social-production-p2p
- [0015](./0015-nix-toolchain.md) Nix flake (holonix main-0.7) for the toolchain
- [0016](./0016-enforced-cleanliness.md) Cleanliness is enforced by CI
- [0017](./0017-own-tauri-shell.md) ~~Own Tauri 2 shell embedding the holochain crate~~ (superseded by 0021)
- [0018](./0018-sweettest-scenarios.md) Scenario tests use Holochain sweettest (Rust), not Tryorama
- [0019](./0019-sveltekit-3.md) SvelteKit 3 with #lib imports
- [0020](./0020-mobile-platform-findings.md) Mobile platform findings (M0 platform check)
- [0021](./0021-holochain-runtime-tauri.md) Use Holochain's official runtime-tauri for the native app (supersedes 0017)
