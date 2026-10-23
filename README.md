# Social Production

An open-source coordination platform for collective projects, shared means of production
and liquid democracy. It has no central database, no admin keys and no server it depends on. It runs on
[Holochain](https://www.holochain.org/): it starts from one cloud seed node and grows into a
network of desktops, Raspberry Pis and phones.

- How it fits together: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- Why it is built this way: [docs/decisions/](docs/decisions/)

## Status (2026-10-23)

**M3 governance (plans, phase returns, project values, links) and M4 chats (project chat, direct and group messages) and notifications are live** on the test network: https://ui-production-333c.up.railway.app
(it runs alongside the current FastAPI site; that site is unaffected).

What works today:
- **@mentions (ADR 0047):** web's mention menu in every chat (project, direct, group), ranked by
  who is closest (this chat, then your other chats, then everyone); `@name` links to their
  profile, and the person mentioned gets a notification that opens the chat on that message.
  A path to one of the app's pages in a message (`/projects/…`) opens as a link, as on web.
- **Notifications (M4d, first part):** web's Notifications page and the bell's unread count:
  phase votes, phase changes and replies in project chats, worked out on each device from the
  projects it has opened; a card opens the chat or the Vote sheet (ADR 0046).
- **Messages (M4c):** web's Messages page. Message someone from their profile or start a direct
  or named group chat; chats list newest first with unread counts, and the send icon counts
  unread messages, as on web. The Public tab lists your
  projects' chats with their unread counts (ADR 0044); a project chat's member count opens its
  members. Group managers rename and add or remove
  people, anyone can leave; the chat menu pins, mutes or deletes a chat; messages can be edited
  and pinned (ADR 0045). Your messages in a direct chat show ✓ when sent and ✓✓ once read
  (ADR 0048). Each chat is
  its own private network; invitations travel encrypted (ADR 0043).
- **Links (M3e):** web's Links tab. A member proposes a link to another project with a reason;
  each project votes on its own half and the link is active once both pass. Either side can
  vote to sever it (ADR 0042).
- **Project values (M3d):** in the Proposal phase members add what the project should achieve and
  rate each value 1–10 (web's value cards); values averaging 5+ become criteria every plan is
  rated on (ADR 0041).
- **Phase changes (M3c):** web's Phase ▾ menu in the Take part tab: advance, close, or return to an
  earlier planning phase, each with a reason that the vote sheet shows. Decisions that left
  members out of the count can be challenged for 48 h (rules only; no screen yet, ADR 0040).
- **Plans (M3b):** in a plan phase, members add a production, distribution, operations or access
  plan through web's plan wizard, rate it on web's criteria and vote on it; the leading approved
  plan lets the project move on (ADR 0039).
- **Project chat (M4a):** every project has a Chat tab with replies and author deletes, polled
  live (ADR 0038).
- **Governance (M3a):** members propose edits, updates and moving to the next phase; others vote in
  the Vote sheet (web's VoteDockControl); a decision applies once 66% of a quorum of members say
  yes. Every node checks the maths, and only full members' votes count (ADR 0033). Leaving the
  Proposal phase also needs 66% support among signals. Votes are final.
- **Usernames** follow web's rules (no spaces); the key tag tells same-named people apart.
- **Projects:** start one through the "+" menu (type → basics → location → tags → overview), and
  find it in the feed (Trending / Most recent, Today / This week / This month / All time) and in
  search. Join and leave, support or oppose, open the member list; the creator edits the charter
  (through proposals since M3). New members can start 2 projects a week; full members have no limit.
- Each project is its own network; the seed node's archivist keeps every project available and
  serves the feed (ADR 0030, 0031). Code-only releases no longer reset anything (ADR 0029).
- **The interface is `web/`'s**: app shell, onboarding, feed cards, project page, create flow,
  sheets, profile header with Vouch, settings (profile, devices, passkey, your key, sign out).
- **Identity (M1):** a 24-word recovery phrase *is* the account, optional passkeys, device pairing
  shown as one person, standing with founders (**thebestdamn**, **BigBird**), 2 vouches → Full.

Measured locally: a new project shows in another person's feed within one archivist round (≤15 s);
signup ≈ 7 s.

Known gaps:
- The M3e deploy (2026-10-20) changed the project DNA, so projects were reset again (people and
  profiles stay).
- Links to events come with events (M5), as do personal-service updates.
- Guests browse the feed signed out but can't open project pages yet; feed cards don't show your
  own signal (see docs/plans/M2-projects.md#as-built-2026-10-19).
- A refused plan rating or vote isn't shown yet (STATUS.md, Known gaps).
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
| **M3** | Governance | ✅ M3a: Proposal/Vote/Outcome for edits, updates, phases ([plan](docs/plans/M3-governance.md)). ✅ M3b: plans. ✅ M3c: returns, challenge rules, reasons. ✅ M3d: project values. ✅ M3e: links between projects | ✅ live |
| M4 | Talk | ✅ M4a: project chat with replies and author deletes, polled live ([plan](docs/plans/M4-talk.md)). ✅ M4c: web's Messages page (direct and group chats, Public tab, group options, chat menu, edits, pins, the send icon's unread count). ✅ M4d first part: web's Notifications page and bell (phase votes, phase changes, chat replies). ✅ @mentions in every chat, with mention notifications and links to the message. Next: more of web's notification kinds as their features land (discussion threads parked: not on web's project pages) | in progress |
| M5 | Work | activities, roles, events, help + service requests, communities/channels | |
| M6 | Platform | moderation + bot reviews, calls, map/places, push relay, software PR/merge flow, board receipts | |
| M7 | Mobile + cutover | Tauri Android/iOS light nodes, multi-seed failover, archivist + feed index on every full node (ADR 0036), Storage setting (light / slice / archive, size limit) and Android help-while-charging (ADR 0037), replace the FastAPI site | |

## License

AGPL-3.0-or-later. Holochain itself is CAL-1.0.
