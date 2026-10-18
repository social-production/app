# M1 — Identity (plan, decided 2026-10-18)

Goal: real people with their own agents, who can prove who vouches for them, on every device.
Each decision below becomes an ADR when it ships.

## Decisions

| Topic | Decision |
|---|---|
| Names | Display name, not unique, shown with a short tag from the agent key ("Johan · 7NFm"). No handle registry. |
| Profile | Display name (required), bio, avatar (≤256 KB, resized in the browser, stored as an entry), home area (free text; map in M6). |
| Signup abuse | Per-IP/device rate limit at the gateway + browser proof-of-work at signup. Hosted agents start as **New** (no vote) regardless; inactive hosted agents are pruned after N days. |
| Recovery | 24-word BIP-39 phrase shown at signup; the user must re-enter 3 random words to continue. |
| Login | Passkey per device. **Fallback:** enter the recovery phrase on any device, which binds a new passkey to the same hosted agent. No email or passwords. |
| Devices | Full device linking in M1: each native-app device has its own agent, joined to one person by signed `DeviceLink` entries; the UI merges them. PWA devices all log in to the person's hosted agent. |
| Standing levels | **New → Full**, or **Paused**. (Warm is dropped.) |
| Full via | **2 vouches from distinct full members**, or **steady activity** (chain 30+ days old, active on 10 distinct days, provable from the own chain). |
| Vouch cap | 5 vouches per voucher per 30 days. |
| Vouch accountability | If someone you vouched for is paused by a bot review, you lose the right to vouch for 90 days (your own standing and vote are kept). |
| Genesis | A named founder list (agent keys) in the directory DNA properties starts as Full. It's transparent, and changing it means a new DNA version. |
| Sync speed | Investigate first-contact latency over the Railway relay (>4 min in M0); target <30 s (gossip tuning; Railway TCP/UDP proxy for the relay's QUIC port). |

## Work outline

1. **directory DNA, integrity**: `Profile`, `DeviceLink`, `Vouch`, `StandingGrant` (cites vouch hashes
   or activity evidence), `VouchSuspension` (cites a bot-review Outcome; the review itself is M6).
   Rules live in `sp_governance::standing` (pure, unit-tested). Founder list in DNA properties.
2. **directory DNA, coordinator**: profile CRUD, vouch, claim standing (assemble evidence), device link
   handshake, standing lookup.
3. **Gateway**: passkey registration and login (WebAuthn), one hosted agent per person (generated from
   the person's seed so the phrase restores it), recovery-phrase restore, proof-of-work + rate limit,
   session per person (replaces the M0 demo agent).
4. **Native app**: seed import/export via the runtime plugin (restore the phrase on desktop),
   DeviceLink pairing.
5. **UI**: signup (passkey + phrase + confirm 3 words), login, restore, profile view/edit, person
   chip "Name · tag", vouch button, standing badge, devices list.
6. **Network**: first-contact latency investigation and fix.
7. **Tests**: sp_governance unit tests for every standing rule; sweettest scenarios (vouch → grant →
   validated on another node; forged grant rejected; cap enforced); Playwright signup/login/restore.
8. Deploy to `social-production-p2p`, then the Grokbot retest sheet.

## Open, decide during build
- Seed derivation: Holochain seed bundle vs. BIP-39 → seed, then agent key (must match what the runtime
  plugin's seed import expects, so a phrase restores the same agent everywhere).
- Prune window for inactive hosted agents (proposal: 60 days with no session).
