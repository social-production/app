# Social Production

An open-source coordination platform for collective projects, shared means of production
and liquid democracy. It has no central database, no admin keys and no server it depends on. It runs on
[Holochain](https://www.holochain.org/): it starts from one cloud seed node and grows into a
network of desktops, Raspberry Pis and phones.

- How it fits together: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- Why it is built this way: [docs/decisions/](docs/decisions/)

## Status (2026-11-01)

**M3 governance (plans, phase returns, project values, links) and M4 chats (project chat, direct and group messages) and notifications are live** on the test network: https://ui-production-333c.up.railway.app
(it runs alongside the current FastAPI site; that site is unaffected).

What works today:
- **Threads and posts (PR #84, 2026-11-01 12:13 am Melbourne, ADR 0070):** web's Post and
  Thread in the + menu; thread and post pages with nested comments, comment votes and web's
  sort; threads in the feed and Search, posts on profiles and Home's personal rows. This reset
  the test network for projects, events, help requests, channels and communities. Live
  `threads.mjs` passed (after the follow-up fix that queues the thread vote with comments).
- **Home scope and updates (PR #83, 2026-10-31 9:38 pm Melbourne):** Home lists the items
  tagged with the channels and communities you joined and your private communities' events,
  as on web. Project and event pages show the newest update on one line under the description,
  with web's Update button beside Edit. Live `closed-communities.mjs` and `governance.mjs`
  passed.
- **Private-community events (PR #82, 2026-10-31 6:47 pm Melbourne, ADR 0069):** web's
  "Private community" event audience: pick one of your private communities; its members see
  the event on the community's page and join without an invite. This reset the test network
  for projects, events, help requests, channels and communities. Live `closed-communities.mjs`
  passed.
- **Personal help-request card (PR #81, 2026-10-31 2:52 pm Melbourne):** help requests in the
  Home feed's personal rows and on profiles use web's personal help-request card (when, where,
  sign-ups). Live `help-requests.mjs` passed.
- **Help request Share menu (PR #80, 2026-10-31 12:11 pm Melbourne):** web's Share button on a
  help request: copy or share the link, or send it to people, who get it in a direct chat and
  as a "shared with you" notification. Live `help-requests.mjs` passed.
- **Search kinds (PR #79, 2026-10-31 1:53 am Melbourne):** web's Search finds channels and
  communities (your private ones too), events and help requests as well as projects and
  people, with web's filter and its icons. Live `search.mjs` passed.
- **Rejoining after a removal (PR #78, 2026-10-30 7:33 pm Melbourne):** someone removed from
  a channel or community who joins again (link, invite or Join) counts as a member at once.
  Live `closed-communities.mjs` passed.
- **Private communities (M5d d, PR #77, 2026-10-30 4:56 pm Melbourne):** web's Openness
  "Private" and Who decides when creating a community (its own network, never listed), the
  lock mark, web's invite panel (link, code, invite by name), "Private community" invites,
  and Who decides in Settings (organizers remove at once; members vote). Live
  `closed-communities.mjs` and `scope-settings.mjs` passed.
- **Scope settings and votes (PR #76, 2026-10-30 2:35 pm Melbourne):** channel and community
  pages have web's gear (Settings: propose a new description; Decisions: open and past votes)
  and web's Vote button while a decision waits for you. Live `scope-settings.mjs` passed.
- **Discover (PR #75, 2026-10-30 12:55 pm Melbourne):** the left rail's Discover button opens
  web's sheet of every channel and community (search, filter, member counts, "Joined"). Live
  `discover.mjs` passed.
- **Tag chips (PR #74, 2026-10-30 10:16 am Melbourne):** channel and community tags on cards
  show web's icons and open their page; "+N" opens web's sheet of every tag. Live
  `projects.mjs` passed.
- **Channels and communities (M5d a–c, ADR 0068, PR #71, 2026-10-29 8:34 pm Melbourne; the
  project network reset):** web's create-channel/community forms, channel and community pages
  with their tagged items, the create flows' scope step (primary channel, more channels and
  communities; web requires a channel tag), and members can vote someone out. Live `help-requests.mjs` and `projects.mjs` passed.
- **Seed-node memory sorted (PR #70, 2026-10-29 3:10 pm Melbourne; ADR 0066):** transparent
  huge pages off for the gateway. Live idle 498 MB at 10 min (was ~790–1000 MB); after a
  browser test run 645 MB (was 1112 MB).
- **Help requests (M5c, ADR 0067, PR #69, 2026-10-29 12:33 pm Melbourne; the project network
  reset):** web's create flow, the request page (roles with Sign up / Leave / Full, votes, chat)
  and its feed card. Live `help-requests.mjs` passed.
- **Event members sheet + allocator change (PR #68, 2026-10-29 10:31 am Melbourne):** private
  events' members sheet splits Organizers and Members, organizers promote members and invite
  people (web's EventMembersPanel); organizer-run events say "Edit event"/"Close event".
  The seed node's jemalloc now also serves C code (ADR 0066); `/readyz` shows `maps`. Live
  smoke: `private-events.mjs` passed. Live idle 781 MB at 10 min (was 941 MB after the last
  deploy), but ~300 MB of anonymous memory is still outside jemalloc (STATUS has the follow-up).
- **The commons (ADR 0065, PR #66, 2026-10-28; the project network reset):** public projects,
  events and their chats share one network instead of one each, so an unused project costs
  rows, not a running network; the seed node's index re-reads only projects with recent
  activity. Private events stay their own networks. Live after the deploy: 1 app / 3 cells idle
  (941 MB, 66 threads); two new accounts with a project and chat add only their own 6 cells.
- **Invite-only and organizer-run events (M5c part 2a, ADR 0061, PR #62, 2026-10-28; the
  project network reset):** web's create-event Audience, Who, Control and Plan steps; private
  events open at `/events/<seed>`, invites arrive under Invites (left rail count and an
  `evt-invite` notification) to accept or decline; organizer-run events start in Activity and
  only organizers change them. Seed-node fixes the same day: SQLite readers capped per
  database (ADR 0062), coordinator updates reach every hosted cell (ADR 0063), and the seed
  node restarts once after an upgrade so old code isn't kept in memory (ADR 0064).
- **The activity button's right rail (2026-10-27, PR #57):** web's panel with your upcoming
  activities, votes you still need to cast (Assess opens the vote), History (ended activities,
  roles you hold, votes you cast) and Cleared cards, plus the button's badge. Since PR #59
  it also lists link votes (opening the Links tab on the request), and a plan vote opens that
  plan's sheet.
- **Activities and roles (M5b, ADR 0056, merged 2026-10-26 17:50 UTC; the project network
  reset, so every test project starts again):** in a project's Activity phase, web's calendar
  with Add activity, the creation wizard, agenda rows, and each activity's sheet: take or leave
  a role, suggest someone, decline a suggestion, share. Past activity: participants check in
  Completed / Uncompleted and rate it 1–5. Notifications for new activities, role sign-ups and
  suggestions (Settings' Roles switch). Project chats take photos and files, and remember what
  you've read on every device.
- **Follows in the feed (ADR 0055):** web's Home / Global scope and audience menus; Home shows
  the projects of people you follow on web's personal cards (profiles use them too); @mention
  suggestions list people you follow first.
- **Follows and Feedback (M5a, ADR 0054, live 2026-10-25 ~20:00 UTC; the directory reset, so
  everyone signed up again):** follow people (Follow / Requested / Following), follower and
  following counts and lists, optional approval with requests in Settings and on your profile,
  follow notifications; web's Feedback page (bugs and suggestions, votes, trending), readable
  signed out.
- **Parity with web (2026-10-25, `docs/plans/PARITY.md`):** the phone More sheet, feed-card
  comment counts and "Latest:" lines, web's onboarding surface and About page, web's Search page
  (people and projects, also for guests), guest profiles and the profile feed.
- **Guests (ADR 0052):** signed out, people open project pages read-only (Details, Take part,
  Chat, Links); joining, supporting or sending shows web's "Sign in" notice.
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
  (ADR 0048). Direct and group chats take photos (shown in the chat, full screen on tap) and
  files up to 10 MB, as on web (ADR 0049; project chats not yet). A direct chat's phone button
  places a voice call with web's call screen (ring, mute, speaker, return pill); the two
  devices signal each other directly and the audio flows between them (ADR 0050). Each call
  leaves web's line in the chat: "Voice call · 4:12", "Call declined", "Missed call" (ADR 0051).
  Each chat is
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
- Guests browse the feed and open project pages read-only (ADR 0052); feed cards
  don't show your own signal (see docs/plans/M2-projects.md#as-built-2026-10-19).
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
| M4 | Talk | ✅ M4a: project chat with replies and author deletes, polled live ([plan](docs/plans/M4-talk.md)). ✅ M4c: web's Messages page (direct and group chats, Public tab, group options, chat menu, edits, pins, read receipts, photos and files, voice calls in direct chats with web's call lines, the send icon's unread count). ✅ M4d first part: web's Notifications page and bell (phase votes, phase changes, chat replies). ✅ @mentions in every chat, with mention notifications and links to the message. Next: more of web's notification kinds as their features land (discussion threads parked: not on web's project pages) | in progress |
| M5 | Work | ✅ M5a: follows, Feedback page, Home scope ([plan](docs/plans/M5-work.md)). ✅ M5b first part: activities, roles, Past activity, activity notifications, project-chat photos. ✅ The right rail (with link and plan votes). ✅ M5c: public and private events, help requests (feed activity cards wait on Region, M6). ✅ M5d: channels and communities, private communities with invites, settings and votes (ADR 0068). Every browser test runs live with test founders. Next: parity pass | in progress |
| M6 | Platform | moderation + bot reviews, map/places, posts and threads, service requests, push relay, software PR/merge flow (asset management and board receipts come after parity) | |
| M7 | Mobile + cutover | Tauri Android/iOS light nodes, multi-seed failover, archivist + feed index on every full node (ADR 0036), Storage setting (light / slice / archive, size limit) and Android help-while-charging (ADR 0037), replace the FastAPI site | |

## License

AGPL-3.0-or-later. Holochain itself is CAL-1.0.
