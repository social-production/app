# Status: where the P2P app stands (updated 2026-11-01 ~1 am Melbourne by a cloud session; see "In flight" and the newest "Done")

Read this first in a new chat, then `CLAUDE.md` (it has a **Cloud sessions** section), `README.md`
(roadmap) and `docs/CODE_RULES.md`. The workspace-level `../STATUS.md` covers web/web-backend.
**The owner is travelling on 2026-10-20/21 and will work from cloud sessions only.** Everything
below is on GitHub; nothing important lives only on the owner's PC.
**Merging:** the owner approved cloud sessions merging a PR once its CI is green (CLAUDE.md).
To start a cloud session, open claude.ai/code on `social-production/app` and say: "Read STATUS.md
and continue from Next."

## Owner decisions (2026-10-19, cloud handoff)
- **Port only from web.** Never invent screens, tabs or features that web doesn't have on that
  page. web's old site is the public repo github.com/social-production/web
  (`git clone --depth 1 https://github.com/social-production/web ../web`); compare with live web
  (read-only) at https://web-production-885ec.up.railway.app, at 390px first.
- **No discussion threads on project or event pages.** Do NOT merge `claude/m4b-threads` (it adds
  an invented Threads tab after Chat). It is parked; its `discussion` zome may be reused later for
  web's channels/communities, but only when those screens are ported from web.
- Cloud sessions may merge a PR to `main` once its CI is green. Merging deploys. Never push
  straight to `main`.
- Cloud work order (done 2026-10-19): (1) PR #1 → merge → smoke → README/STATUS docs PR; (2) Take part layout
  (`claude/take-part-layout`); (3) `claude/m3c-reverts`: merge main, check it invented nothing
  (its app-only ChallengeList must be flagged to the owner, since web-backend has no challenge
  window), open a PR.

## Owner direction (2026-10-25)
- **The owner steers less until parity; the cloud session steers the ship.** Decide, record the
  decision here or in an ADR, and keep going. The owner will review once the app matches web.
- Secrets can wait: everything on the P2P network is temporary test data with no real users
  until the FastAPI site is replaced.
- UI gaps (e.g. phone "More" opens the left rail instead of web's `MobileMoreSheet` with
  profile/settings/about) get fixed during the parity pass, from web's existing screens.

## Owner direction (2026-10-31)
- **Parity with web is the goal; asset management is not part of it** (platform assets, attached
  assets, the platform board, asset-use requests come later). The parity list is
  `docs/plans/PARITY.md` ("Whole web pages the app doesn't have yet").
- Owner noticed updates on project and event pages look different from web's: fixed in PR #83.

## Direction (2026-10-28): the commons, ADR 0065 (plan `docs/plans/COMMONS.md`)
Owner: "we could have tens of thousands of projects… major changes can still be made… you have
control". One network per project cannot reach that (each costs two databases, a gossip space and
a cell on every archiver). Decided: public spaces become records in the project DNA's base cell
(one sharded network); private spaces stay clones of the same DNA; the index follows changes via
time-bucket links. **This comes before M5d and the rest of M5c** (communities/channels would
otherwise be more clones). Phase 0 (measure) first, then integrity, coordinators/gateway, UI.

## Railway outage 2026-10-29 ~05–07:35 UTC: resolved
Every Railway URL returned "Application not found". Cause: the owner's Railway usage limit; they
raised it and all three services came back by 07:36 UTC (seed node restarted, 533 MB at 41 s).
If every URL 404s again, ask the owner to check Railway usage/billing first.

## In flight: the follow-up PR after #84 (the thread vote shares the write queue; these docs).

## Next: the parity pass (`docs/plans/PARITY.md`)
Candidates: "copy link" in the card ⋮ menu (web's ⋮ is `ReportControl` +
`ReportMenu`; copy link alone would be a one-item menu until moderation), "Plan became
leading" (web-backend sends `prj-plan-lead` to the new leader's author when a plan vote
changes the leader; deriving it needs the time the lead changed, which the plan list doesn't
give: work that out first). Then the big pieces in PARITY's "Whole web pages the app doesn't
have yet" (moderation, map/places, service requests, software PR flow; posts and threads are
done, with their later parts listed in PARITY);
asset management is not part of parity (owner, 2026-10-31). Each starts with its ADR.

## Done 2026-11-01: threads and posts (ADR 0070), PR #84 merged 13:13 UTC (12:13 am Melbourne), live ~13:35 UTC
- **Live after merge:** CI took 72 min (slowest shard 3). The seed node restarted at ~13:35
  UTC; `/readyz` 603 MB at 5 min. **The project network was reset** (integrity change): live
  test projects, events, help requests, channels and communities from before ~13:35 UTC are
  gone. Live `threads.mjs` **failed** at the reply: the thread's vote (`setSignal`) and the
  queued comment writes raced on one chain ("source chain head has moved"; local runs were
  fast enough to miss it). Fixed in the follow-up PR: `DiscussionState.vote` goes through
  `services/chat.ts`'s `queued`. With that fix, a production build against the live gateway
  passed `threads.mjs`. The test now prints the page's error note when the reply never shows.
web's standalone discussion items, the first of PARITY's "Whole web pages the app doesn't have
yet". **Integrity change in the project DNA** (new charter modes + `MessageVotes` link in the
discussion zome): merging resets the project network (test data only).
- **DNA:** new crate `sp_discussion` (modes `thread`/`post`, vote tags, post titles). Charter
  accepts the modes; the discussion integrity zome gains `MessageVotes` (message → voter, tag
  up/down, only on a chat message). Coordinators: no phases (stage "Thread"/"Post"), no
  proposals, no activities; `discussion::vote_message` / `message_votes` (guests may read
  votes: gateway allowlist).
- **Gateway:** posts only on creator-filtered pages (Home's personal rows, profiles); threads
  and posts rank by votes. Index tests moved to `node/index_tests.rs` (size rule).
- **UI:** `features/projects/discussions/` (DiscussionPage for `/threads/<id>` and
  `/posts/<id>`, CreateThread, CreatePost, DiscussionTools: Share + Sort),
  `components/discussion/` (DiscussionPanel, DiscussionComment, CommentComposer), ThreadCard,
  PersonalPostCard, Threads in Search, Post + Thread first in the create menu.
  `ScopeTagsStep` gained web's thread option (primary tag type). `CreateSheet` hides the step
  rail for one step and takes `submittingLabel`/`canSubmit`. Styles via PORTS
  (`discussion.css`, cards/create appends). `scripts/port-styles.py` now keeps web's
  `:global(html:has(…))` selectors unscoped (it used to prefix them, making them dead).
- **Tests:** scenario `threads` (no lifecycle, no proposals, nested comments, one vote each,
  votes only on comments); `tests/e2e/threads.mjs`.
- **Checked (cloud session, 2026-10-31):** zomes build + pack; `cargo clippy` (project zomes,
  gateway) and `cargo fmt`; `cargo test -p sp_discussion`, gateway index tests; scenarios
  `threads` and `help_requests` **passed** (`cloud-scenarios`-style run, ~12 min each); UI
  check + lint, boundaries, sizes; against a local seed node (`cloud-seed.sh`):
  `threads.mjs`, `closed-communities.mjs`, `events.mjs`, `search.mjs` (now expects Threads in the filter; its events check reads links
  only, since an empty network shows "No results") and `help-requests.mjs` **passed**. Not run
  locally: the other scenarios (CI runs them all).
- Messages' Public tab leaves thread/post comments out (web lists projects, events and help
  requests there); notification cards say "Thread"/"Post".


## Done 2026-10-31: Home's joined channels and communities + updates as on web, PR #83 merged 10:38 UTC (9:38 pm Melbourne), live ~10:58 UTC
- **Live after merge (~11:05 UTC):** `closed-communities.mjs` (community event on Home) and
  `governance.mjs` (update line + Update button) passed against the live UI. CI took 75 min
  (slowest shard 5). `/readyz` 750 MB at 4.5 min uptime. No network reset (no DNA change).
**Updates (owner, 2026-10-31):** web's current `ProjectOverviewHeader`/`EventOverviewHeader`
show only the newest update, one line under the description ("Update: …", a leading "update:"
stripped), with an **Update** button in the dock beside Edit; web's `ProjectUpdatesSection`
is no longer imported anywhere. The app had kept that old list (+ the line, whose template
whitespace showed through `white-space: pre-line`). Now: `ContextPanel` one line,
`ProjectDock` Update button, `UpdatesSection.svelte` and its PORTS rows removed;
`closedCommunities.ts` cards' "Latest" was the oldest update (fixed: newest first).
`governance.mjs` checks the line and the button and **passed** locally (one earlier run timed
out later on, at the feed card, and passed on the rerun).

web-backend's home feed (`services/feeds/builder.py`, `selects.py`): items tagged with any
channel or community you joined, plus `private_community` events of communities you're in.
- **Gateway (no DNA change):** `/index/projects?tag=` takes several tags, comma-separated (any
  of them; `TAGS_MAX` 500); `Index::page` takes `tags: &[&str]` (unit test covers two tags).
- **UI:** `features/projects/homeFeedState.svelte.ts`: on Home, `myScopes()` → the public
  scopes' tags go to the index; private communities' events come from `communityEventCards`
  (filtered by the search box, not by the time window) and join the public rows.
- **Tests:** `tests/e2e/closed-communities.mjs` now ends with two seeing the community event
  on Home; `follows.mjs`'s "no public rows" assertion says why (bob joined nothing).
- **Checked:** UI check + lint, boundaries, sizes; `cargo fmt`, `cargo test -p gateway --lib
  index` (3 passed); `closed-communities.mjs` and `follows.mjs` **passed** against a local seed
  node (`cloud-seed.sh`). Not run: scenarios (no zome change; CI runs them).

## Done 2026-10-31: events for a private community (ADR 0069), PR #82 merged 07:47 UTC (6:47 pm Melbourne), live ~08:04 UTC
- **Live after merge (08:06 UTC):** `closed-communities.mjs` passed against the live UI
  (community event found and joined). CI took 77 min (slowest shard 5). Deploy statuses green
  ~08:03 UTC; the seed node restarted once (ADR 0064); `/readyz` 649 MB at 3 min uptime.
  **The project network was reset** (integrity change): live test projects, events, help
  requests, channels and communities from before ~08:04 UTC are gone.
web's third event audience, **Private community**. **Integrity change in the project DNA**
(new `ScopeItems` link).
- **DNA:** `dnas/project/zomes/integrity/project/src/scope_item.rs` (community space → lister,
  tag = a private event's seed; only in a private clone, on a community, by the event's
  creator). Coordinator `add_scope_item` / `scope_items` (`coordinator/project/src/scope_items.rs`).
  Scenario `scopes` (closed-community test) checks listing, refusals and reading.
- **UI:** Audience step has web's three options; Who step picks one of your private
  communities (+ organizers); the event is a private event tagged `@<slug>` and listed on the
  community (`services/events.ts` `createEvent`, `listOnCommunity`); the private community's
  page shows listed events to members (`closedCommunities.ts` `communityEventCards`,
  `scopeState.svelte.ts`). `audienceOf` reports `private_community`; the members sheet treats
  it as private, as web does.
- `tests/e2e/closed-communities.mjs` now ends with one creating a community event and two
  finding it on the community page and joining. It needs the new DNA: run it against a local
  seed node or live.
- **Checked:** zomes build + pack, `cargo clippy -p project -p project_integrity -p scenarios
  --all-targets`, cargo fmt, UI check/lint, boundaries, sizes; scenario `scopes` **passed**
  (both tests, 23 min); `closed-communities.mjs` **passed** against a local seed node
  (`cloud-seed.sh`) with the new DNA. Not run: the other scenarios (CI runs them).
- **Disk lesson (2026-10-31):** `cloud-seed.sh` builds the gateway with debug info unless you
  export `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0` (as the scenarios do); without
  them it rebuilds holochain separately and filled the disk to 2 GB free. With them it reuses
  the scenario build and starts in ~3 min. This session also removed a stale `target/release`
  (3.8 GB).

## Done 2026-10-31: help requests on web's personal card, PR #81 merged 03:52 UTC (2:52 pm Melbourne), live ~03:53 UTC
- **Live after merge:** `help-requests.mjs` passed against the live UI (profile → personal
  card included). CI took 60 min (slowest shard 4). `/readyz` 711 MB at 2.5 h uptime.
**PR #81** (branch `claude/social-production-handoff-dsam61`): UI only. Personal-feed rows
(Home's followed/joined items) and profile feeds now pick their card by mode
(`components/cards/PersonalSpaceCard.svelte`, web's `PersonalFeedCard`): a help request gets
`PersonalHelpRequestCard.svelte` (web's: tags, the asker's identity row, when · where, sign-ups,
votes and comments, the clock); everything else keeps `PersonalActivityCard`. Styles via PORTS
(`cards.css` `.personal-help-request-card`). `tests/e2e/help-requests.mjs` now opens the
asker's profile from the feed card's author link (it reads the href: clicking raced the feed's
re-render) and checks the personal card. **Checked:** UI check/lint, boundaries, sizes;
`help-requests.mjs` **passed** with a production build against the live gateway. Not run:
scenarios (no Rust change).

## Done 2026-10-31: the help request's Share menu, PR #80 merged 01:11 UTC (12:11 pm Melbourne), live ~01:13 UTC
- **Live after merge:** `help-requests.mjs` passed against the live UI (share → notification →
  open). CI took 59 min (slowest shard 4). UI-only: the seed node didn't restart (`/readyz`
  682 MB at 10 h uptime).
UI only (no Rust change, no network reset). web's Share menu left of the help request's vote
strip, signed in (`features/projects/help/HelpRolesDock.svelte`). `ShareUserMenu` gained
web's share-outside half (`components/overlay/ShareOutside.svelte`: Share via…, Copy link,
share links; styles were already in `share.css`) and `compact`. Sending posts shares.py's
"Shared a help request: …\nOpen help request: <url>" into your direct chat with them
(`services/helpRequests.ts` `shareHelpRequest`); their device derives web's `hr-share`
notification from it (`domain/share.ts`, read in `services/notifications.ts` `fromChat`),
labelled "Help request", which opens the request. `span.share-sheet` is `display: contents`
(additions.css) so web's 56px `.share-shell` is the dock's flex child. ADR 0067 "Sharing".
`tests/e2e/help-requests.mjs` now shares with the helper and opens the notification.
**Checked:** UI check/lint, boundaries, sizes; `help-requests.mjs` **passed** (share → the
helper's notification → opens the request) with a production build against the live gateway;
390px screenshots of the dock and the sheet match web's layout. Not run: scenarios (no Rust
change).

## Done 2026-10-30: Search finds channels, communities, events and help requests, PR #79 merged 14:53 UTC (1:53 am Melbourne, Oct 31), live ~14:55 UTC
- **Live after merge (14:55 UTC):** `search.mjs` passed. CI took 60 min (slowest shard 4).
  UI-only merges don't restart the seed node: `/readyz` 951 MB at 6.2 h uptime, 34 apps /
  118 cells.
UI only (no Rust change, no network reset). `services/search.ts` follows web-backend's
`search_documents`: channels and communities first (`listScopes`, plus your private
communities from `myClosedCommunities`; handle/slug contains a query of 2+ letters, ranked by
`domain/search.ts` `scopeMatchRank` = web-backend's `_scope_match_rank`), then people, then
the project feed's items as Project / Event / Help request (a one-kind filter reads 50 and
keeps that kind; the index has no mode filter). The filter has web's options in web's order
for the kinds the app has, with icons (`showOptionIcons`). New `tests/e2e/search.mjs`
(guest, 390px) **passed** with a production build against the live gateway. **Checked:** UI
check/lint, boundaries, sizes. Not run: scenarios (no Rust change).

## Done 2026-10-30: rejoin after a removal (`people::join`), PR #78 merged 08:33 UTC (7:33 pm Melbourne), live ~08:45 UTC
- **Live after merge (08:50 UTC):** `closed-communities.mjs` passed (remove → rejoin with the
  link). CI took 75 min (slowest shard 5). `/readyz` 995 MB at 213 s, right after the test.
Coordinator-only change in the project DNA (no integrity change, so no network reset):
`people::join(removed)` (`dnas/project/zomes/coordinator/project/src/people.rs`, called from
`lib.rs`'s `join` with `applied()?.removed`) keeps your live `Members` link unless it predates
your latest passed removal; then it deletes it and creates a new one. The UI's `joinScope`
(leave, then join) is gone: callers use `joinProject(id, true)`. Scenario `scopes` gained
remove → join → member again. ADR 0068 "Rejoining after a removal" updated. Also carries the
earlier README/STATUS docs commits. **PR #78** opened 07:20 UTC (6:20 pm Melbourne). **Checked:** zomes build + pack, `cargo clippy -p project`,
scenario `scopes` **passed** (both tests, 39 min in this cloud session), UI check/lint,
boundaries, sizes. Not run: the other scenarios (CI runs them), browser tests.

## Done 2026-10-30: M5d part (d), private (closed) communities, PR #77 merged 05:56 UTC (4:56 pm Melbourne), live ~06:08 UTC
- **Live after merge (06:11 UTC):** `closed-communities.mjs` and `scope-settings.mjs` passed.
  CI took 58 min (slowest shard 1). The seed node restarted twice after the deploy (ADR 0064's
  restart-once); `/readyz` showed 1067 MB at 149 s, right after both test runs.
UI only (no Rust change, so no network reset). Details and deviations: ADR 0068 "Part (d)".
- **Create:** `CreateScope.svelte` has web's Openness + Who decides; Private →
  `services/closedCommunities.ts` `createClosedCommunity` (a clone, `private.`/`organized.`
  seed, closed charter, self-invite through the inbox).
- **Finding them:** `myClosedCommunities()` reads your project clones; `services/scopes.ts`
  `findScope` (a private one you're in wins its slug), `myScopes` (left rail), Discover
  ("· Private"). The route passes `page.url.hash`; `#invite=<seed>` opens a community by seed.
- **Scope page:** lock mark; web's invite popover (`components/IconPopoverMenu.svelte`,
  `features/projects/scopes/CommunityInvitePanel.svelte`): link + code + invite by name for
  members, "Join with invite" for others. No feed for private ones (nothing can tag them).
- **Settings:** `ScopeWhoDecides.svelte` (organizers: add organizer, remove at once with a
  confirm; collaborative: propose a removal). Organizers' description edits say Save.
- **Invites/notifications:** `pendingInvites` has `kind`; `/invites` says "Private community"
  and opens it; notifications use web-backend's `community-invite` kind.
- **Rejoin fix (UI):** `services/projects.ts` `joinScope` leaves, then joins, because the
  zome's `join` keeps a removed member's stale `Members` link. **Follow-up:** fix
  `people::join` in the project coordinator (compare with your latest removal), then drop
  the extra `leave`.
- **Checked:** UI check/lint, boundaries, sizes; new `tests/e2e/closed-communities.mjs`
  (create organizer-run → invite a new account by name → accept in Invites → remove →
  rejoin with the link) **passed twice** with a production build against the live gateway
  (`VITE_SP_GATEWAYS=<live seed node> npm run build -w ui`, then `npx vite preview --port
  4174` from `ui/`; **rebuilding kills a running preview, restart it**). One earlier run
  passed every step but caught a `ClientClosedWithPendingRequests` page error while the test
  navigated away mid-request; not seen again. Not run: scenarios (no Rust change).
- **Live dev founders have no profile names** ("Unnamed"), so tests that search people by
  name must sign up a named account (as this test and `private-events.mjs` do).
- **Disk:** this session found `/tmp` full of ~350 stale `nix-shell.*` dirs (8 GB) and
  `target/debug` (11 GB); it removed `target/debug`. Delete the `/tmp/nix-shell.*` dirs if
  the disk fills again (the session's sandbox refused to sweep `/tmp`).

## Done 2026-10-30: scenario shards by time + scope Settings/Vote sheets, PR #76 merged 03:35 UTC (2:35 pm Melbourne), live ~03:47 UTC
- **Live after merge (03:51 UTC):** `scope-settings.mjs` and `discover.mjs` passed. `/readyz`
  744 MB, 28 apps / 84 cells, 3 min after the deploy.
- **Shard times on PR #76:** 31–57 min, except shard 5 (events, links, scopes) at 74 min:
  `scopes` took 38 min there vs 25 on PR #75. A file's time varies by up to ~50% between runs,
  so the table's balance is approximate; still far under 120 (was 105–118 for shard 0).
PR #75's shard 0 took 104.6 of its 120 min (PR #74's: 118.5): files went to shards by name
order, and `governance` (45 min), `private_events` (40) and `activities` (30) all landed in
shard 0. `scripts/check-scenarios.sh` now deals files heaviest first to the lightest shard,
using each file's CI minutes (a table in the script, from PR #75's logs; a new file counts 15
until measured). Dry run: every shard ~47–48 min of tests (+ ~3 min build). **When a file's
time changes a lot, update its minutes there.**
- **Same PR: scope Settings and Vote sheets (M5d part e).** The scope page has web's gear
  (`features/projects/scopes/ScopeSettingsSheet.svelte`: Settings tab with the read-only name,
  description and "Propose change"; Decisions tab with open/past decisions and Yes/No) and, while
  a decision waits for you, web's Vote button (`ScopeVoteSheet.svelte`, one at a time).
  `scopeGovernance.svelte.ts` lists the scope's proposals (closing settled ones as projects do),
  proposes description edits and votes. `ProposalKind` gained `RemoveMember` in the UI types.
  Styles via PORTS (`scopes.css` `.scope-settings`, `.scope-vote`). Differences from web are in
  ADR 0068 "Deviations in the settings sheets" (no rename, no picture, final votes, removal in d).
  New browser test `tests/e2e/scope-settings.mjs` (two founders, 390px: propose → Vote button →
  Yes → description changes → Decisions lists it as passed) **passed** with a production build
  against the live gateway. Checked: UI check/lint, boundaries, sizes.

## Done 2026-10-30: Discover sheet (M5d part b, last piece), PR #75 merged 01:55 UTC (12:55 pm Melbourne), live ~02:07 UTC
- **Live after merge (02:08 UTC):** `discover.mjs` passed. `/readyz` 707 MB, 28 apps / 84 cells,
  39 s after the seed node's restart-once-after-upgrade (ADR 0064).
web's `ScopeDiscoverSheet`: the left rail's **Discover** button (it went to `/` before) opens
`features/shell/ScopeDiscoverSheet.svelte`: every channel and community from `/index/scopes`
(`listScopes(q, 200)`), search, web's All/Channels/Communities filter (`IconMenuButton` gained
`icon` per option + `showOptionIcons`), member counts, a "Joined" badge (from `myScopes`); a row
opens the scope page and closes the phone rails. Styles ported via PORTS (`scopes.css`
`.scope-discover`). `myScopesLoop.ts` has `scopeHref`/`railLinks` (kept AppShell under 300 lines).
No "· Private" yet: closed communities don't exist (part d).
- New browser test `tests/e2e/discover.mjs` (guest, 390px; `lib.mjs` now exports
  `ensureTestChannel`). **Passed** with a production build of this branch (`VITE_SP_GATEWAYS=
  <live seed node> npm run build -w ui`, `npm run preview -w ui -- --port 4174`) against the live
  gateway; screenshot matches live web's Discover sheet at 390px.
- Checked: UI check/lint, boundaries, sizes. Not run: scenarios (no Rust change).
- Live `/readyz` before this PR (00:05 UTC Oct 30, 39 min after PR #74's deploy): RSS 624 MB,
  28 apps.

## Done 2026-10-29: tag chips link to their channel or community (M5d part c, last piece), PR #74 merged 23:16 UTC (10:16 am Melbourne, Oct 30), live 23:29 UTC
web's `TagList` (plain) in `components/cards/FeedCardTop.svelte`: each `#channel`/`@community`
tag shows its icon (`ScopeGlyph`; `#platform` → platform icon, `/platform`) and opens its page;
free tags from before channels still search. "+N" now opens web's `ScopeSheet`
(`components/overlay/ScopeSheet.svelte`, styles ported into `scopes.css` under `.scope-sheet`)
instead of expanding inline. Helpers: `domain/scopes.ts` `tagRef`, `components/cards/tagLinks.ts`.
- `tests/e2e/projects.mjs` tags its project with its own channel plus `#testing` (`lib.mjs`
  `pickScopeTag`, `tagTestChannel(page, label)`), then at 390px checks the icon, the link, the
  "+1" sheet and that a row opens `/channels/testing`.
- Checked: UI check/lint/boundaries/sizes; `projects.mjs` **passed** with a production build
  of this branch (`VITE_SP_GATEWAYS=<live seed node> npm run build -w ui`, `npm run preview -w
  ui`) against the live gateway; 390px screenshots of feed and sheet match web. **Use the
  preview, not `npm run dev`, for browser tests:** the dev server's unbundled modules made
  Chromium fail reloads with `ERR_INSUFFICIENT_RESOURCES`. Not run: scenarios (no Rust change).
- **Live after merge (23:31 UTC):** `projects.mjs` passed (icons, links, "+1" sheet). `/readyz`
  668 MB at 2.5 min (seed node redeployed with the merge).

## Done 2026-10-29: web's required channel tag + test fixes, PR #73 merged 13:16 UTC (12:16 am Melbourne, Oct 30)
PR #73 (branch `claude/social-production-handoff-dsam61`): the `projects.mjs` race fix below, plus
**web's scope requirement**: a project needs a channel tag; a public event or a help request a
channel or community tag (`CreateState.scopeNeeded()`, overridden for events; help's
`problems.scope`). The Continue button stays disabled with "Still needed: a channel tag."
- Browser tests: `tests/e2e/lib.mjs` `continueSteps(page, n)` clicks Continue and tags the
  shared `#testing` channel on a scope step (`tagTestChannel`; founder one creates it once per
  run, so New members' 2-a-week listing allowance isn't spent on it). 13 tests use it.
- Checked: UI check/lint/boundaries/sizes; live `events`, `activities` (with the helper);
  the local UI (`VITE_SP_GATEWAYS=<live seed node> npm run dev -w ui -- --port 5199`) against
  the live gateway: the gate (disabled → enabled), `events`, `help-requests` (one unexplained
  timeout on the first run, passed on re-run). Not run: the other 10 updated tests.
- **Live after merge (13:17 UTC):** `projects`, `events`, `help-requests` passed. `/readyz`
  742 MB, 218 threads after ~10 browser-test runs since the 09:55 UTC restart.

## Done 2026-10-29: channel picker, PR #72 merged 11:24 UTC (10:24 pm Melbourne) + test fix
- `services/scopes.ts` `listScopes` adds this device's just-created scopes (`justCreated`) until
  the index lists them (a new channel takes ~1–2.5 min to reach `/index/scopes` live).
- The live `projects.mjs` failures were a **test race**, not the app: the tag loop filled the box
  and called `option.count()` before the async search answered, then cleared it. It now waits
  for the option (5 s per try) and starts from the channel page (no reload). **Live
  `projects.mjs` passed** (11:40 UTC). `private-events.mjs` has the same `count()` pattern for
  the people search; it passes live, but fix it the same way if it flakes.

## Done 2026-10-29: M5d channels and communities, parts (a)–(c) (ADR 0068), PR #71
**PR #71** merged 09:34 UTC (8:34 pm Melbourne); seed node live 09:55 UTC. It reset the
project network only (directory `uhC0k-zll3y…` and conversation `uhC0kUmCMHl…` unchanged).
- **(a) DNA:** `crates/sp_scope` (handles, `#channel`/`@community` tags, join policies, reserved
  slugs); project integrity accepts `channel`/`community` charters (title = handle, subtype =
  join policy; a closed community only in a private-seed clone, reusing private events'
  `private.`/`organized.` seeds) and `ProposalKind::RemoveMember` (scopes only; no handle
  edits); governance folds removals into `Applied.removed`; the project view leaves removed
  members out. Scenario `scopes` passes (2 tests; the removal needs a 4-member quorum of 3).
- **Gateway:** `/index/projects` leaves scopes out and takes `tag=#food`; `/index/scopes?q=`
  lists them, earliest listing per handle. Channels/communities still count toward a New
  member's 2 listings a week (directory integrity counts every listing; ADR 0068 deviation,
  fix at the next directory reset).
- **(b) UI:** `/create/channel`, `/create/community` (web's forms; open communities only for
  now), `/channels/<slug>`, `/communities/<slug>` (`features/projects/scopes/ScopePage.svelte`:
  header with join/members, description, the tagged items), create menu "Surfaces" section,
  left rail lists the scopes you joined (`stores/myScopes`, `features/shell/myScopesLoop.ts`).
  `components/cards/SpaceCard.svelte` picks the card by mode for every feed.
- **(c) Tagging:** web's scope step (`ScopeTagsStep`: primary channel, then more channels and
  communities) replaces free tags in the project, event and help-request flows.
  `tests/e2e/projects.mjs` now creates a channel, tags the project and finds it on the
  channel page.
- **Not yet:** web's required channel tag (13 browser tests create projects: add a shared
  "ensure a channel" helper first), tag chips with web's scope icons linking to scope pages,
  Discover sheet, closed communities + invites (d), settings/votes sheets and removal UI (e).
- **Checked locally:** release seed node, `projects` (channel → tagged project → channel page),
  `help-requests`, `events`, `private-events` all passed; scenario `scopes` passed.
- **Live smoke:** `help-requests.mjs` passed. `projects.mjs` failed twice at "channel never
  appeared in the tag search": the search works (an indexed channel shows), but a new channel
  took over ~2 min to reach `/index/scopes` on the freshly reset node. Fixed in the follow-up
  above. `/readyz` 569 MB at 2 min, 611 MB after the runs (4 apps / 12 cells).

## Done 2026-10-29: memory sorted, PR #70 merged 04:10 UTC (3:10 pm Melbourne), live
**Transparent huge pages off (ADR 0066 addendum).** Railway's kernel has THP `always`; every
thread stack and jemalloc extent took 2 MB pages. The gateway now calls
`prctl(PR_SET_THP_DISABLE)` before its runtime starts (`KEEP_THP=1` skips it).
| Live `/readyz` | RSS | outside jemalloc | thread-stack maps |
|---|---|---|---|
| Before #70, 2.4 h, 5 apps | 1002 MB | ~375 MB | 350 MB |
| After #70, 10 min, 5 apps | **498 MB** | ~0 | 63 MB |
| After a live `private-events.mjs` | **645 MB** (was 1112 MB for the same run) | ~0 | 104 MB |
`maps.anonHugeMb` is 2 MB. Live smoke: `private-events.mjs` passed. Watch `/readyz` as data
grows; growth should now follow live data (jemalloc `heap.residentMb`), not threads.

## Done 2026-10-29: help requests (ADR 0067), PR #69 merged 01:33 UTC (12:33 pm Melbourne), live
**The project network reset.** Live: `help-requests.mjs` passed (after the test fix above);
the index card carries `help {neededAt, endsAt, signupCount, slotsNeeded}`. `/readyz` 10 min
after the deploy: RSS 789 MB, 1 app / 3 cells, no 64 MB (glibc) mappings.
What it is: see "2. Help requests" below and ADR 0067.

**1. Done: PR #68 merged 23:31 UTC 28 Oct (10:31 am Melbourne), all three services deployed
(seed node live ~23:53 UTC).** jemalloc for C code (ADR 0066) + web's event members sheet.
- **Live result, not the full win:** `/readyz` at 10 min: RSS 781 MB (after the commons deploy
  it was 941 MB at 5 min), jemalloc resident 359 MB, `maps.anonMb` 662 MB, wasmer code 66 MB.
  So ~300 MB of anonymous memory is still outside jemalloc on the 8-CPU node, though locally
  (4 CPUs) the gap was ~0. After one live `private-events.mjs` run: RSS 1112 MB, anon 959 vs
  jemalloc 528 (gap ~430 MB and growing with use). **Next:** PR #69 adds `maps.anonBySize`
  (RSS by mapping size: 2–8 MB = thread stacks, 64 MB = glibc arena heaps, ≥1 GB = wasm
  memories). Read it live after #69 deploys to see what the gap is, then fix that.
- Live smoke: `private-events.mjs` passed (promotes the invitee in the members sheet).
- What #68 changed: see ADR 0066 and the PR; edit sheet titles/copy and event phase-menu copy
  follow web; "Remove organizer" not done (needs revocable grants, an integrity change).

**2. Help requests (ADR 0067): PR #69, merged (see above).**
- **DNA:** a help request is a commons space with charter mode `help-request` (new crate
  `sp_help`; `sp_core`/`sp_governance` untouched, so directory and conversation keep their
  hashes) plus one `Activity` written by the asker (integrity: only the space's creator adds it
  there). `ActivityRole.description` (serde default). Coordinators: the view gives it no
  phases ("Help request" stage), governance refuses proposals, activities let only the asker
  write it; activity ratings moved to `activities/src/rate.rs` (write.rs was over 300 lines).
- **Gateway:** cards carry `help {neededAt, endsAt, signupCount, slotsNeeded}`
  (`node/cards.rs`, moved out of archivist.rs); help requests rank as Voted; every card's
  ranking now counts comments (as web's formula does).
- **UI:** `/create/help-request` (web's six steps; tags until channels exist), `/help-requests/
  <id>` (`features/projects/help/`: web's Details with roles Sign up / Leave / Full and the vote
  strip, Chat with web's wording), `HelpRequestCard` in the feed, "Help request" in the create
  menu, `components/cards/spaceHref.ts` picks a space's page by mode. Styles ported via PORTS
  (HelpRequestCard, CreateHelpRequestPage, HelpRequestDetailPage, OverviewHeader, RolesSection).
  Event chats now use web's "Event chat" wording. `additions.css` undoes a leaked global
  `.identity-copy` rule (port-styles keeps selectors that merely start with the root name).
- **Checked:** scenario `help_requests` passed (17 min); browser `help-requests.mjs` passed on
  a release seed node; 390px compared with live web's help-request page.
- **Not done:** web's Share menu on the request, the required channel/community tag (M5d),
  `PersonalHelpRequestCard`, help-request notifications.

## Done 2026-10-28: the commons, PR #66 merged 13:43 UTC (12:43 am Melbourne, Oct 29), live
PR #65 (docs, `/readyz` heap + cells, ADR 0065) merged 10:15 UTC; Railway's build of it failed
in a Railway incident ("Builds failing in US West"); PR #66's deploy built fine and carries both.
Live smoke: `/readyz` 941 MB / 66 threads / 1 app, 3 cells at 5 min; `public-chats.mjs` against
live passed (2 accounts, a project, its chat): 3 apps, 9 cells, 1154 MB, no project clones.
**Next:** watch `/readyz` as test data grows (it should no longer step up per project); persist
index cards in `gateway.sqlite`; Phase 4 (private clones cold when idle); then back to M5c/M5d.
What PR #66 changed (ADR 0065 "As built"):
- **Project DNA (reset):** every record names its space (the charter's create hash); links hang
  off the space; `recent.<UTC hour>` links mark activity. Private events stay clones.
- **Coordinators:** calls take `{space, input}`, or a bare input in a private clone
  (`space_scope` crate; `discussion/src/scope.rs`). New project externs: `space`, `touch`,
  `my_spaces` (memberships + chats written in), `active_spaces`; discussion `my_chat_spaces`.
  Links name a commons space `<creator>.space.<id>`. Joining needs the charter locally.
- **Gateway:** the archivist reads every listed space in its one commons cell (no clones) and
  re-reads only new/recently written ones (full pass every ~15 min); guest reads the same.
  `DB_MAX_READERS` default 4 → 16: with one shared database, 4 readers stalled reads for 85 s
  (ADR 0062 amended). **If `DB_MAX_READERS` is set on Railway, it should go.**
- **UI:** listings keep their ids; `createProject` founds the space in the commons cell and
  lists `<me>.space.<hash>`; `projectCell` returns the commons cell tagged with the space
  (`holo/spaces.ts`), `callCell` wraps calls; rail/notifications/Public tab use `my_spaces`.
- **Checked:** fmt, clippy, workspace unit tests, UI check/lint, boundaries, sizes; scenarios
  discussion/events/private_events/challenges locally; every main browser test on a local seed
  node (accounts, projects, chat, events, private-events, governance, links, guests, rail,
  notifications, public-chats, activities). `memory-projects` (debug build): 8 new projects
  moved RSS 1255 → 1297 MB; 10 projects indexed with 24 cells for 8 accounts (no clones).

## Next for the seed node's memory (open, 2026-10-28)
- 2026-10-29: allocator stats are in `/readyz` (`heap`, `maps`); the gap was glibc arenas
  used by C code, fixed by ADR 0066 (see In flight). The bullets below predate it.
- Live costs more per account and project than local (~3.5 MB per project at idle locally,
  ADR 0064): live idle went 887 → 1414 MB across a day of test runs. Add allocator stats to
  `/readyz` (jemalloc allocated vs resident vs retained, plus app and cell counts) to see
  whether it is live data or allocator overhead on the 8-CPU container, then decide between
  the ADR 0064 steps (index on change instead of every 15 s; cold archivist clones).
- The owner asked for projects not to run hot unless opened (2026-10-28): ADR 0064 holds the
  measured plan; build step 1 (index on change) first.

## Done 2026-10-28: restart once after upgrades (ADR 0064) + CI in 6 shards, PR #64 merged (08:07 UTC / 7:07 pm Melbourne), seed node live 08:24 UTC
- **Memory finding:** after PR #62 (network reset) live idle was ~1.51 GB; PR #63's plain restart
  brought it to **~0.89 GB** (8 CPUs, 88 threads). ~0.6 GB came from the conductor starting the
  old version's apps before `remove_stale_apps` uninstalled them (compiled zomes never freed,
  ADR 0053). Fix on this branch: after removing old apps the gateway exits once and Railway's
  `ON_FAILURE` restarts it clean (`RESTART_AFTER_UPGRADE=0` disables; health check 300 s).
  Tested locally with `scripts/kept-seed.sh`: v1 → v2 exits once, the next start serves.
- **Cold projects (owner's question):** measured locally, hot vs every app disabled: ~527 vs
  ~491 MB idle (10 projects, 31 cells), threads 150 vs 17. Projects cost ~3.5 MB each at idle,
  so cold storage is a CPU/scale step, not today's memory fix. ADR 0064 records the order:
  index on change instead of every 15 s, then cold archivist clones; not sleeping people's apps.
- **Growth under use levels off (live, 05:30–05:50 UTC):** settled 1282 MB → +95 after a round
  of `private-events` + `events` → +37 after a second round (1414 MB); threads 102 → 294 while
  busy, falling back when idle. Each round adds accounts and two events, so growth now follows
  new data rather than leaking. Cold projects (ADR 0064) are the lever as data grows.
- **CI: 6 scenario shards now.** PR #64's shard 3 (challenges, governance, messages, reverts) hit
  the 120-min job timeout (95–97 min on #62/#63); `ci.yml` runs 6 shards.
- CI with 6 shards: `check` 5 min, shards 41–70 min (was up to 120 with 5).
- Live after the deploy (a plain restart): `shell.mjs` passed; `/readyz` 1164 MB at 158 s,
  **1414 MB settled at 7 min** (143 threads). Higher than this morning's 887 MB because today's
  test runs added many accounts and events (the archivist rejoins every project at start).
- **The restart path itself is untested live** (this deploy didn't change the hApp version):
  after the next integrity change, check that `/readyz` uptime resets twice and idle sits near
  a plain restart.

## Done 2026-10-28: coordinators updated per cell (ADR 0063), PR #63 merged (04:01 UTC / 3:01 pm Melbourne), seed node live 04:14 UTC
- Fixed live private events ("That isn't a chat."): Holochain 0.7 keeps a ribosome per cell;
  the gateway now updates every cell at startup, after `ensure_hosted` and for new clones.
- Live smoke 04:30–04:45 UTC: `private-events.mjs`, `events.mjs` (feed card too) and `shell.mjs`
  all passed. `/readyz`: idle 887 MB → 1189 after private-events (2 accounts, 1 event) → 1386
  after events → 1328 after shell (threads 88 → 196 → 111). Part is the one-time first load of
  the project zomes after a restart (~110 MB in ADR 0053); **next: watch whether repeated runs
  level off** (run the same files again and compare), since growth under use is the open part.

## Done 2026-10-28: M5c part 2a, invite-only events and organizers (ADR 0061) + SQLite reader cap (ADR 0062), PR #62 merged (00:55 UTC / 11:55 am Melbourne), seed node live 01:09 UTC
**Merging resets the project network** (integrity change; directory and conversation DNA
hashes unchanged: directory has no changed deps, conversation `uhC0kUmCMHl…` as on main).
Done on the branch (pushed):
1. [x] `sp_event::Audience` from the seed (`<creator>.private.<nonce>` / `.organized.`), tests.
2. [x] Project integrity: `Organizer { agent, grant }` + `Organizers` link; `organizer:
   Option<ActionHash>` on `Proposal` and `Activity` (serde default); organized events:
   organizer-only proposals/activities, vote-less passed outcomes, no values/importance/
   signals, no return to Proposal (`integrity/project/src/organizer.rs`).
3. [x] Coordinators: governance `decide_now` closes organizer proposals, skips the signal gate
   and the recount, newest approved plan leads (`plans::leader`); project `add_organizer`,
   `my_organizer_role`, view `audience`/`governance`/`organizers`/`am_organizer`; activities
   cite the grant; messages inbox accepts private-event seeds.
4. [x] Scenario `private_events` (2 tests) passes locally (~15 min for the organizer test).
5. [x] UI: create steps (Audience, Who, Control, Plan; `eventCreateState.svelte.ts`,
   `domain/eventCreate.ts`, `components/create/{EventAudiencePanel,PeoplePicker,EventPlanStep}`),
   private events open at `/events/<seed>` (`isPrivateSeed`, `cellOf`), `/invites`
   (`features/invites`, `services/events.ts` pending/answer, declines in localStorage), left
   rail Invites row + count, `evt-invite` notification (+ "Shares and invites" setting),
   PrivateMark, leave confirm, non-organizers can't propose in organized events.
   [x] browser tests on a local release seed node: `private-events.mjs` (create invite-only
   organizer-run event → opens in Activity → invitee accepts under Invites) and `events.mjs`
   (now clicks Continue 4×: the public path gained web's Audience step) both pass.
   Lessons: event plans need a place, so the up-front plan writes "TBD" when the location is
   decided later (web-backend does too); the first Invites read joins the event's network and
   takes a while; after many runs, project creation hung again until `cloud-seed.sh` restarted.
6. [x] PR #62 merged when CI was green (check 5 min, shards 24–97 min); live smoke: `shell.mjs`
   passed; `private-events.mjs` hit the stale coordinator (see In flight).
**Seed-node memory (ADR 0062, same branch):** per project the conductor opens two SQLite
DBs, and sqlx runs each connection on its own thread with its own page cache; Holochain's
default pool is 2 × CPUs (≥ 8) + 1 per DB. The gateway now sets `DB_MAX_READERS` (default 4 →
5 connections per DB). Local release runs (`MEM_PROJECTS=6 bash scripts/e2e-local.sh
tests/e2e/memory-projects.mjs`): ~16 MB per project before, ~7 MB after; sqlx threads followed
the setting. `/readyz` now has `threads` and `cpus`: **after the deploy, read them live** (if
`cpus` is large the live saving is bigger) and re-sample `rssMb` across a test run. Next if
growth persists: whether the archivist needs a full cell per project.
**Not in this part (next):** the event page's members panel for private events (web's
EventMembersPanel: Organizers section, "Promote to organizer", "Invite +"), "Edit event"/Save
wording for organized events, revoking organizers, activities limited to the plan's days,
private events in the feed for their members, `private_community` (M5d).

## Done 2026-10-28: seed-node memory (jemalloc), busy-conductor retry, test founders live (ADR 0060), PR #61 merged (16:19 UTC 27 Oct / 3:19 am Melbourne), seed node live 16:39 UTC; ALL 18 browser test files passed live
**Directory reset with this merge** (founders list changed): everyone signs up again.
- **Memory:** the gateway uses jemalloc (feature `jemalloc`, on in the Docker build and
  `RELEASE=1` dev seed; off in debug builds, where jemalloc's configure fails under Nix
  hardening). Idle: local 1148 → ~700 MB; **live idle 698 MB** (was 1.1–1.4 GB).
- **Still growing with projects:** live, all 18 files took it 698 → 1407 MB (links rerun:
  1495). Big steps come with projects (`projects.mjs` +270 MB, each `links.mjs` run ~+80 MB for
  2 projects), so the cost is per project cell (the archivist joins every project; the creator
  and members hold it too), not per account. **Next memory step:** profile one cell's cost
  (SQLite page caches and connection pools per cell DB, kitsune space per DNA) with the
  jemalloc recipe (ADR 0053 STATUS entry); see whether the archivist needs a full cell per
  project or a lighter read path.
- **Busy conductor:** creating a project/chat cell retried on SQLite 517 "database is
  locked" (`ui/src/lib/holo/cells.ts`). This was the "project creation hangs" in local runs.
- **Test founders on the test network:** `tests/e2e/fixtures/dev-founders.json` identities
  are founders in `dnas/directory/dna.yaml` (remove at launch). **Every browser test now runs
  live**, no owner accounts needed:
  `for t in <files>; do node tests/e2e/$t.mjs https://ui-production-333c.up.railway.app; done`.
  Live run 16:41–17:13 UTC: all passed; `links.mjs` failed once (cause not captured) and
  passed on rerun. Locally `governance.mjs` also timed out once and passed on rerun: watch both.
- The identity scenario now counts the founders in `dna.yaml` instead of expecting 2.
- **Next:** M5c part 2 (private events: secret seed + invites, organizer-controlled
  governance; activities limited to the plan's days), then help requests (own ADR), then M5d
  (communities/channels + coordinates on listings, ADR 0058, in one directory reset). Do the
  per-cell memory profiling alongside.

## Owner direction (2026-10-27)
- **No real users; launch only on the Holochain system**, at parity, after a UX pass
  (`docs/plans/UX.md`). The FastAPI site is a prototype: it gets wiped with all its data and
  users; nothing is migrated or handed over, and its secrets don't need rolling.
- **Full nodes:** desktop users choose whether they are full; "full" always stays available;
  full nodes hold the feed index too; sharding waits for thousands of users (ADR 0036/0037
  updates).
- **Map:** ADR 0058 (OpenFreeMap, Photon via gateways, approximate by default, viewer location
  on the device). **Types evolve through plans** and personal services go, after parity
  (ADR 0059). Until then don't add rules that fix the type at creation.
- Estimate given: parity in about 1–2 weeks at the current pace.

## Done 2026-10-27: M5c events, first part (ADR 0057), PR #60 merged (11:21 UTC / 10:21 pm Melbourne), seed node live 11:33 UTC; live `shell.mjs` and `guests.mjs` passed
**Merging resets the project AND directory networks** (integrity changes): everyone on the test
network signs up again; projects, follows and chats from before become unreachable. The
conversation DNA hash is unchanged (`hc dna hash`: directory `uhC0kzkuwcU…`, project
`uhC0kOPCCpG…`, conversation `uhC0kUmCMHl…` as on main).
- **Decision (ADR 0057):** an event is a project-DNA clone whose charter mode is `"event"`,
  using the app's phase ids (Proposal `phase-1`, Event Plan `phase-2`, Activity `phase-5`,
  Closed `phase-7`). Event rules are in `sp_governance` (mode `EVENT`: labels, phases, returns
  to any earlier phase, `PlanKind::Event`, web's event rubric) and a new crate `sp_event`
  (schedule shape, charter kind). `sp_core` untouched.
- **Event plans** carry `schedule: Option<EventSchedule>` (mode any-day/date/range, dates,
  times, stored in UTC as web does) and need a place. The event page shows the leading
  approved plan's time and place (`domain/eventSchedule.ts`).
- **Directory:** a listing's `mode` is a project mode (checked as before) or any lowercase
  kind (`event`, later `help-request`, `community`, `channel`) with shape checks only, so
  later kinds need no directory reset.
- **UI from web:** "Event" in the + menu, `/create/event` (public path: Basics, Tags,
  Location, Overview; web's Audience step waits on private events), `EventCard` in the feed,
  `/events/[id]` (the project page with web's event copy, "Join event"), the event plan
  wizard (date or range, times, place), phase copy, rail/notification/search links to
  `/events/`. Styles via PORTS (`CreateEventPage`, `EventCard`). `domain/plans.ts` split:
  the wizard form is `domain/planForm.ts`.
- **Tests:** new scenario `events` (2 tests, ~10 min; it must settle after each plan vote or
  the last voter can't close the plan); `plans`, `values`, `projects` re-run and pass. New
  browser test `tests/e2e/events.mjs` (create, join, Event Plan, dated plan approved, time
  and place on the page, Activity, event card in the feed) passed; `governance`, `rail`,
  `projects` passed.
- **Lesson:** on the local release seed node, project creation started hanging (heading never
  appears) after an aborted browser test, even without "database is locked" in the log.
  Restarting `cloud-seed.sh` fixed it each time. Also: while scenarios run, browser tests
  can time out on the 4-core container; run them on an idle machine.
- Not run here: the full `check-all.sh` (CI runs it).
- CI: `check` 15 min, shards 39–110 min (shard 3 now holds challenges, governance, messages
  and values; move to 6 shards if it nears 120). Deploy: ui/bootstrap at once, seed node 12
  min. Live 12:20–12:30 UTC: `shell.mjs`, `guests.mjs` passed; `/readyz` 1455 MB at start,
  1662 MB after. The live test network was reset by this merge (sign up again).
- **Was next after merge:** smoke live (`shell.mjs`, `guests.mjs`, and `events.mjs` needs the dev
  founders, so local only); README Status. Then M5c part 2: private events (secret seed +
  invites, ADR 0043 style) and organizer-controlled governance; activities limited to the
  plan's days; help requests (own ADR).

## Done 2026-10-27: rail link votes + plan votes open the plan's sheet, PR #59 merged (06:48 UTC / 5:48 pm Melbourne), seed node live 07:00 UTC; live `rail.mjs` and `shell.mjs` passed
PARITY "Rail gaps". **UI only, no zome or gateway change**, so merging resets nothing.
- Feed/profile activity cards (the old Next) turned out to wait on M6: web's
  `ProjectActivityCard` comes only from web-backend's Region feed (`feeds/region.py`, with
  distance); web's feed has no plan cards; event cards come with M5c. PARITY updated.
- Rail: web-backend's link rows ("Link vote · <other project>" / "Sever link · …", kicker
  "Vote · link" / "Vote · link sever"): this project's half of each open request you haven't
  voted on (`governance.link_sides`, one more read per joined project per minute), newest 9;
  the other project's title from `get_listing` in the directory (cached; no project network
  joined). Cast link votes go to History (12, both kinds together).
- Project page: `?tab=links&linkRequest=<half id>` opens the Links tab, expands the request
  (or the active link whose sever vote it is), scrolls to it and flashes web's
  `request-highlight`. `?open=vote&voteKind=plan&voteTarget=…` selects the plan's phase and
  opens that plan's sheet (Assess is inline there); closing drops the link.
- Tests: rail helpers moved to `tests/e2e/lib.mjs`; `links.mjs` has founder two open B's link
  vote from the rail; `governance.mjs` has Bob open the plan from his rail.
- Checked: `npm run check`/`lint`, sizes, boundaries; local release seed node
  (`RELEASE=1 bash scripts/cloud-seed.sh`): `links`, `governance`, `rail` all passed. A first
  `governance` run failed on a strict-mode locator (the row's "Assess and approve this plan"
  also matched the Assess button; now `exact: true`); after that aborted run founder one's
  writes kept failing with SQLite "database is locked" (code 517) until the seed node was
  restarted. **Lesson:** if project creation hangs locally after an aborted test, check the
  page for "database is locked" and restart `cloud-seed.sh`.
- No scenario changes (no zome change). CI: `check` 5 min, shards 35–76 min. Deploy: seed
  node 12 min after the merge; the live UI bundle has the new rail rows ("Sever link").
  Live 07:01–07:15 UTC: `rail.mjs` and `shell.mjs` passed on the first run (`links`/`governance`
  need the dev founders, so they ran locally only). `/readyz` 604 MB at start, 1060 MB after.
- Nothing is in flight. Next: M5c (events and help requests), starting with its ADR; read
  web's events screens (`features/events/**`) and web-backend's event tables first.

## Done 2026-10-27: "suggestion declined" notifications, PR #58 merged (04:31 UTC / 3:31 pm Melbourne), seed node live 04:44 UTC; live `activities.mjs` and `shell.mjs` passed
M5b leftover. **Coordinator only** (`activities` zome's view gains `RoleView.declined`), so the
DNA hashes are unchanged (`hc dna hash` before/after `build-happ.sh`: project `uhC0kkykG2Y…`)
and merging resets nothing; the gateway upgrades hosted coordinators on start.
- UI: `domain/activityNotifications.ts` adds web's "<role> suggestion declined" / "The person
  you suggested declined this role." card for the suggester (`prj-role-suggest`, Roles switch).
- Checked: clippy `-D warnings` (activities, scenarios), fmt; `npm run check`/`lint`, sizes,
  boundaries; scenario `activities` (2 passed, 14 min, now asserts the decline); local release
  seed node: `activities.mjs` (now waits for Ann's declined card) and `notifications.mjs` passed.
- CI: `check` 5 min, shards 36–73 min. Deploy: ui 1 min, seed node 13 min after the merge.
  Live 04:45–04:49 UTC: `activities.mjs` (Ann got the declined card) and `shell.mjs` passed
  on the first run; `/readyz` 587 MB at start, 1253 MB after (4 accounts, a photo).
- Nothing is in flight. Next: feed/profile activity cards (PARITY "Event, activity and plan
  cards"), then the rail gaps (link votes; plan Assess → the plan's sheet), then M5c.

## Done 2026-10-27: web's right rail (the activity button), PR #57 merged (02:18 UTC / 1:18 pm Melbourne), seed node live 02:40 UTC; live `rail.mjs`, `shell.mjs`, `guests.mjs` passed
PARITY "Activity (calendar) button" and M5b's "right rail's activity". **UI only, no zome or
gateway change**, so merging resets nothing.
- `domain/activityRail.ts` + `activityRailHistory.ts` (pure): web-backend's
  `bootstrap/activity_rail.py` and `activity_rail_history.py` rules and limits, from the
  projects open on this node (`services/activityRail.ts`: `get_project`, `list_activities`,
  `list_proposals`, `list_plans` per joined project, as notifications do, ADR 0046).
  `stores/activityRail.svelte.ts` keeps cleared/seen ids under web's localStorage keys;
  `features/shell/activityRailLoop.ts` refreshes it every 60 s while visible and on opening.
- UI from web: `features/shell/RightRailPanel.svelte` + `components/rail/*` (sections, rows,
  History with its Show filter, Cleared with Restore / Restore all, `SurfaceTypeLabel`); the
  top bar's badge; the first desktop load opens the rail when something needs doing.
  Styles: `styles/rail.css` via PORTS, rooted at `.right-rail` (web's left rail also uses
  `.rail-panel`; Svelte scoped them apart, our global CSS must not).
- Vote links `?open=vote&voteKind=edit|update&voteTarget=…` now open the Details tab's vote
  sheet (web's place for them); phase votes still open Take part's. `listPlans` takes a seed.
- Decided (owner review in PARITY): personal services' activities (`phase-1`) are listed too;
  web-backend checks `phase-5` only. Gaps: link votes, plan Assess opens the tab not the plan.
- Checked: `npm run check`/`lint`, sizes, boundaries; local release seed node
  (`RELEASE=1 bash scripts/cloud-seed.sh`): new `tests/e2e/rail.mjs` (signed-out rail text
  matches live web's; activity row, badge, clear/restore, open → sheet; Bob's edit vote →
  Assess → vote → History), `shell`, `governance`; `notifications` timed out once creating a
  project after `governance` had loaded the founders' node, then passed alone. Watch whether
  the rail's extra reads (4 calls per joined project per minute) slow busy accounts.
- Live at session start (00:50 UTC): `/readyz` uptime 4.8 h (PR #56's redeploy), `rssMb` 641.
- CI: `check` 5 min, shards 35–85 min. Deploy: ui 1 min after the merge, seed node 22 min.
  Live smoke 02:41–02:46 UTC: `rail`, `shell`, `guests` all passed on the first run.
  `/readyz` 577 MB at start, 1037 MB five minutes later after those tests (~5 accounts).
- Then: the "suggestion declined" card (done, PR #58); feed/profile activity cards; rail gaps: link votes, plan Assess → the plan's sheet).

## Done 2026-10-27: docs + CI in 5 shards, PR #56 merged (19:33 UTC / 6:33 am Melbourne)
- README Status/roadmap for M5a/M5b; CI runs the scenarios in 5 shards (CLAUDE.md too).
- **PR #55 deployed:** merged 17:50 UTC; ui + bootstrap live 17:52 UTC, seed node 18:16 UTC
  (26 min; this one succeeded, so PR #54's gateway is live too). Live smoke 18:21–18:29 UTC
  all passed on the first run: `shell.mjs`, `guests.mjs`, `follows.mjs` (Home personal cards,
  mention order), `activities.mjs` (activity, roles, suggestion, project-chat photo, Past
  activity check-in and rating). `/readyz` 1333 MB at start, 1672 MB after those tests
  (~9 accounts): keep sampling it.
- CI with 5 shards: `check` 5 min, shards 46–63 min (with 4 the worst was 102). Nothing is
  in flight; continue from Next.

## Done 2026-10-26: M5b activities and roles, first part (ADR 0056), PR #55 merged (17:50 UTC / 4:50 am Melbourne 27 Oct)
**The project network reset with this merge** (integrity change): every live test project starts again;
people, follows and chats are untouched (directory and conversation hashes unchanged).
- `crates/sp_activity` (limits, time locks, `fill`: who holds each role when two take the last
  place at once: earliest action wins, one role per person, leavers lapse until it ends).
- Project integrity `activity.rs`: `Activity` (roles embedded, web's linked plan stage as a
  string id), `RoleSuggestion`, `SuggestionDecline`, `ActivityRating`; taking a role is an
  `Assignments` link (deleted to leave), refused after the activity ends.
- New coordinator zome `activities` (project DNA): `create_activity` (Activity phase:
  phase-5, or phase-1 for a personal service by its creator; starts in the future),
  `list_activities`, `take_role`, `leave_role`, `suggest_role`, `decline_suggestion`,
  `rate_activity`; guests read `list_activities` through the gateway.
- Ride-alongs in the discussion zome: photos and files in project chats (`Chunk`,
  `upload_chunk`, `get_chunk`; guests read chunks) and a private `ReadMark` (`mark_read`,
  `read_at`), so the Public tab's unread counts follow you across devices.
- UI from web: calendar card + Add activity, creation wizard (place is a plain label or
  Online), agenda rows, the activity sheet (roles Take/Leave, Suggest someone, Decline, Share),
  and Past activity (web's ActivityHistorySection: participants check in Completed /
  Uncompleted and rate 1–5; Community shows completion and ratings). Check-ins are an
  `ActivityCompletion` entry (`check_in`), added before merging so the reset happens once.
- Tests: scenario `activities` (2 tests, ~28 min locally: it waits for a short activity to
  end); `discussion` scenario re-run; browser `activities.mjs` (new: create, take → Full,
  leave, suggest/decline, a photo in the project chat) plus chat, public-chats, attachments,
  projects, governance (timed out once under load, then passed), notifications, guests,
  mentions, all against a local release seed node.
- `activities.mjs` also runs a two-minute activity into Past activity (check-in + rating) and
  checks the role-taken and suggestion notifications.
- Activity notifications (role taken, suggested for a role, new activity) and Settings' Roles
  switch are in too; `notifications.mjs` still passes.
- Owner review (2026-10-26): feed cards had lost their dividers (the app wraps each card's
  `.surface` in a scoping div, so web's `.stack .surface:last-child` rule matched every card;
  fixed in `additions.css`), and a personal card with no tags showed its stage alone above the
  author; it now sits on the author's row (web's `header-end`). **Lesson:** compare computed
  styles with live web, not only screenshots of one card.
- Next (no reset needed): the "suggestion declined" card, feed/profile activity cards, the
  right rail's activity (PARITY; the rail shipped in PR #57).

## Done 2026-10-26: follows in mentions and the feed (ADR 0047 update, ADR 0055), PR #54 merged (15:05 UTC / 2:05 am Melbourne 27 Oct)
STATUS Next (M5a leftovers). **No zome change** (gateway + UI), so merging resets nothing.
- **Mention ranking** (web-backend `people_suggestions.py`): people you follow, then your
  followers (accepted only). Circles now: this chat, following, followers, your other chats'
  members, everyone (`domain/mentions.ts` `MentionPool`, `services/follows.ts` `followCircles`).
- **Home scope (ADR 0055):** web's scope menu (Home / Global; Region waits on M6) and audience
  menu (Following only / Following + popular, signed in), with web's `combineFeeds` default
  on. Home = joined channels/communities (none until M5d) merged with the projects of people
  you follow (`domain/homeFeed.ts` `mergeHomeFeed`, `features/projects/homeFeedState.svelte.ts`).
  Personal rows use web's `PersonalActivityCard` + `PersonalFeedIdentity` (new, styles via
  PORTS), and so does the profile feed. Gateway: `/index/projects?creator=` takes a
  comma-separated list (≤500). Scope/audience kept per device (`sp_feed_scope`,
  `sp_home_audience`). "Following + popular" changes nothing until posts/threads exist.
- **Fixed** `scripts/port-styles.py`: `:global(...)` with nested parentheses is unwrapped by
  paren matching, so `detail.css`'s project-overview signal-row rules (dropped before, with a
  lightningcss build warning) now apply as on web. Regenerating all PORTS now changes only
  `detail.css` (+ the new cards block) after prettier: the other ported files match today's web.
- `screens/` (test screenshots) is now in `.gitignore`: the first commit had added some by
  mistake; they're removed again on this branch.
- Tests: `tests/e2e/follows.mjs` checks Ann's menu offers Cat (followed) before Bob (a
  follower, sorts first), Bob's Home shows Ann's project as a personal card and Global shows it
  once as a public card; `guests.mjs` reads the profile feed's personal cards.
- Checked: `npm run check`/`lint`, sizes, boundaries; gateway clippy `-D warnings` (release),
  fmt, gateway tests (new multi-creator test); against a local release seed node
  (`RELEASE=1 bash scripts/cloud-seed.sh`, restarted with this gateway): `follows`, `guests`,
  `shell`, `projects`, `governance`, `mentions` all passed. No scenario changes (no zome change).
- Live at start of session (~13:00 UTC 2026-10-26): `/readyz` uptime 15.4 h (no restart since
  the M5a deploy), `rssMb` 629.
- CI: `check` 5 min, shards 43–73 min.
- **Deploy: ui and bootstrap went live (~15:06 UTC); the seed-node deploy FAILED at 15:48 UTC**
  ("Deployment failed" after 43 min, no logs from a cloud session). The old seed node (PR #52's
  gateway) kept running. Until a seed-node deploy succeeds, the live Home feed shows nothing
  for someone who follows two or more people (the old gateway reads `creator=` as one key).
  Owner: look at the seed-node build log in Railway (Deployments → the failed one); the next
  merge (#55) redeploys anyway. Then smoke `follows.mjs`, `guests.mjs`, `shell.mjs` live.

## Done 2026-10-25: M5a follows + feedback (ADR 0054), PR #52 merged (19:32 UTC / 6:32 am Melbourne), seed node live 19:57 UTC
**This reset the directory network: everyone on the test network signs up again**
(projects and chats tied to old agents become unreachable, as after earlier directory changes).
Project and conversation DNA hashes are unchanged (`hc dna hash`: project `uhC0kJTRl5t…`;
conversation's zomes untouched); directory is now `uhC0kA14W4Y…`.
- Directory integrity: `Follows`, `Followers`, `FollowAccepts`, `FollowRejects`,
  `RequiresFollowApproval` links; `Feedback` entry with `AllFeedback` / `FeedbackVotes` links.
  Status rule in the new pure crate `crates/sp_follow` (coordinator-only, no DNA hash impact).
- Identity coordinator: `follow`, `unfollow`, `accept_follow`, `reject_follow`,
  `set_follow_approval`, `follow_view`, `follow_requests`, `create_feedback`, `list_feedback`,
  `get_feedback`, `vote_feedback`; the caller's own follows and votes read from their chain.
  Guests read `follow_view` / `list_feedback` / `get_feedback` (gateway `node/public.rs`).
- UI from web: profile Follow button, follower/following counts + people sheets, requests card
  on your own profile; Settings Follow requests + Privacy (approval switch); follow
  notifications (with Accept / Decline on the card) and the Follows switch; the Feedback page.
- Checked here: clippy `-D warnings` (identity, identity_integrity, sp_follow), `sp_follow`
  tests, `npm run check`/`lint`, sizes, boundaries; hApp build + DNA hashes; scenarios
  `follows` (2), `feedback` (1), `identity` (4); browser tests on a local release seed node:
  `follows.mjs` (new), `guests`, `notifications`, `shell`, `accounts`. CI green on `c9b30f1`.
- Live (19:59–20:08 UTC): `follows.mjs`, `shell.mjs`, `guests.mjs` all passed. Seed node up since
  19:57 UTC with no restart; `rssMb` 1192 at start, 1424 after those tests (9 accounts): higher
  than the ~1.2 GB seen after earlier smokes, so keep sampling `/readyz` next session.
- Known, not from this branch: the UI build warned that `detail.css` had a multi-line
  `:global(` the port script didn't unwrap; fixed 2026-10-26 (PR #54).
- Still to do in M5: mention ranking by follows (ADR 0047), the personal feed / Home scope,
  then M5b (activities and roles, one project reset).

## Open 2026-10-25: seed-node crash (12:30 UTC), diagnostics deployed with PR #50 (merged ~14:17 UTC, live ~14:46 UTC)
Railway reported "Deploy Crashed" for seed-node right after PR #49's deploy, during the live
smoke (`projects.mjs` saw its websocket close with 1006 at ~12:31 UTC). Railway restarted it;
it has answered since. **Cause not found yet**: not reproduced locally (same tests on main,
release build, no panic, 1.2→1.4 GB). The owner pasted log lines: `sqlcipher_mlock: mlock()
returned -1 errno=12` (SQLCipher can't lock key memory in Railway's container: harmless noise)
and "Railway rate limit of 500 logs/sec reached", from `NoPeersForLocation` publish warnings
(hosted people's publish retries in networks where nobody else holds that part).
- This branch: the gateway keeps `publish_dht_ops_workflow` at `error` whatever `RUST_LOG`
  says (locally: 0 such lines, 97 lines for a full test run), so the crash line can't be
  drowned out next time; `/readyz` now returns `uptimeSecs` and `rssMb` so a session can see
  restarts and memory without Railway access.
- After PR #50 (live ~14:46 UTC): `/readyz` uptime rose without a reset through the live
  smoke; memory 811 MB at start, ~870 MB idle after 10 min, 1.2 GB after `projects`, `guests`
  and `shell` created ~6 accounts. `projects.mjs` and `shell.mjs` passed; `guests.mjs` timed
  out once waiting for Ann's new project page (no restart; output not captured), then passed
  in full with a failure screenshot hook. Watch for slow project creation on the live node.
- Next if it crashes again: ask the owner for the last ~50 lines before the crash in the
  deploy log (look for `panicked at`, `memory allocation`, `killed`), and watch `/readyz`
  `uptimeSecs` resets and `rssMb` from a cloud session.

## Done 2026-10-25: Settings appearance + notification switches, PR #49 merged (~11:50 UTC / 10:50 pm Melbourne), deployed ~12:29 UTC; live `shell.mjs` and `guests.mjs` passed, `projects.mjs` passed every step and then saw the seed-node crash above
From `docs/plans/PARITY.md` (Settings, compared with web's `SettingsPage.svelte` source):
- Appearance: type size, theme (light/dark/system), accent colour, font, per device under web's
  storage keys (`stores/appearance.svelte.ts`, applied by the shell; `app.html` also restores
  the type size before first paint). The CSS was already ported in `app.css`.
- Notifications: web's switches for Comments, Votes needed, Phase finished (per device,
  `stores/notifications.svelte.ts` filters what the bell and page show).
- Tests: `shell.mjs` sets Dark + Blue and checks they survive a reload; `notifications.mjs`
  switches Votes needed off (card hidden) and on again.
- Checked: `npm run check`/`lint`, sizes, boundaries; against a local release seed node:
  shell, accounts, notifications. No Rust changed.

## Done 2026-10-25: Search page, guest profiles, profile feed, PR #47 merged (~08:35 UTC / 7:35 pm Melbourne), deployed ~08:56 UTC; live `shell.mjs`, `guests.mjs`, `projects.mjs` passed
From `docs/plans/PARITY.md`:
- web's Search page (`features/search/SearchPage.svelte`, `components/search/*`,
  `services/search.ts`, `styles/search.css`): people + projects, live suggestions, kind filter
  (All / Projects / Users), web's empty-query chips. People come from a new gateway index
  (`node/people.rs`, `GET /index/people?q=`, the archivist re-reads profiles every minute), so
  guests search too. The "+" follows web's `isFeedDiscoveryPath` (not on search any more).
- Guests open profiles (`/profile/[agent]`, `AccountGate guests`; no actions, no device count).
- Profile feed: web's toolbar and stack with the person's projects (`features/projects/
  ProfileFeed.svelte`, composed by the profile routes; `/index/projects?creator=`).
- Tests: `shell.mjs` searches a person as a guest and opens the profile; `guests.mjs` opens the
  author's profile and checks it lists only their projects; `projects.mjs` searches by tag on
  the Search page; `governance.mjs` reads Latest from the feed sorted Most recent.
- Checked: `npm run check`/`lint`, sizes, boundaries; gateway fmt/clippy/tests (release, new
  people + creator-filter tests); against a local release seed node: shell, guests, projects,
  governance, accounts.
- CI: `check` 5 min, shards 37–81 min. After deploy: `/index/people?q=` answers live; `shell.mjs`
  and `projects.mjs` passed live; `guests.mjs` timed out once a minute after the seed node
  restarted (output not kept), then passed in full: the cold-node pattern again.

## Done 2026-10-25: parity batch, PR #46 merged (~06:52 UTC / 5:52 pm Melbourne), deployed ~07:09 UTC; live `shell.mjs`, `guests.mjs`, `projects.mjs` passed
From `docs/plans/PARITY.md` (the new checklist; compared with live web at 390px):
- Phone More opens web's `MobileMoreSheet` (`components/shell/MobileMoreSheet.svelte`,
  `shell/more-sheet.css`), pixel-identical with web.
- Feed cards: web's comments `CountPill` (opens `?tab=chat`) and "Latest: …" line. The
  gateway index card gained `latestUpdate` and `commentCount` (archivist reads
  `list_messages` per project per round; counts up to 200).
- Onboarding is web's auth surface (brand only; `Topbar auth`), web's heading; the signed-out
  intro shows only on the feed.
- About: web's page and copy (`features/about/*`, `styles/about.css`); owner review flagged in
  PARITY (the copy describes web).
- Tests: new `tests/e2e/shell.mjs`; `guests.mjs` checks the comments pill, `governance.mjs`
  the Latest line; `lib.mjs` `signUp`/`phraseSignIn` now wait for the redirect off
  /onboarding (onboarding hides the bottom nav, so clicking it right after sign-in raced).
- Checked: `npm run check`/`lint`, sizes, boundaries; gateway clippy/tests (release); against a
  local release seed node: shell, guests, governance, projects, accounts, messages, chat,
  notifications (the last two once hit "database is locked" on a long-running local node;
  passed after a restart, see How to work).

## Done 2026-10-25: seed-node out-of-memory fix (ADR 0053), PR #45 merged (~04:57 UTC / 3:57 pm Melbourne), deployed ~05:25 UTC; live `projects.mjs` and `guests.mjs` passed
Railway reported "Deploy Ran Out of Memory" for seed-node (2026-10-25). **Found and fixed:**
Holochain 0.7.1 evicts a DNA's compiled zomes after every new cell's genesis (and after 1 h
idle); each reload adds the machine code to wasmer's shared engine, which never frees it. So
every sign-up, chat and opened project leaked ~100 MB per cell (~350 MB per new account,
measured on a release build).
- Fix: `vendor/holochain` (patched 0.7.1, `[patch.crates-io]`, row in `vendor/README.md`): no
  eviction at genesis, no time-to-idle. **No DNA or hApp change**: merging resets nothing.
- Measured (release, `RELEASE=1 bash scripts/dev-seed.sh`): 3 accounts +110 MB then +8 MB for 3
  more (was +0.9–1.2 GB each time); 7 browser test files moved the node 1.1 → 1.4 GB.
- Tried and dropped: sleeping idle hosted apps (disable after 2 h, wake on resume). It worked
  but freed nothing (the leak isn't owned by apps) and would make sleeping people unreachable.
  If idle apps ever cost CPU/network, the design is in ADR 0053's "Considered".
- How it was found (reuse for any memory problem): release build; RSS from `/proc/<pid>/status`
  per step; `smaps` showed anonymous heap; jemalloc built with `--enable-prof` through Nix
  (`nix build --impure --expr '…jemalloc.overrideAttrs (o: { configureFlags = o.configureFlags
  ++ ["--enable-prof"]; })'`), `LD_PRELOAD` on the gateway only (a Nix library preloaded into
  system binaries breaks them), `MALLOC_CONF=prof:true,lg_prof_sample:19,lg_prof_interval:30,
  prof_prefix:…`, then `jeprof --text --inuse_space [--focus=…] target/release/gateway <heap>`.
- Checked in the cloud session: release gateway build (no warnings), against the patched local
  seed node: `projects`, `guests`, `chat`, `messages`, `governance`, `attachments`, `calls`
  browser tests all passed. Not run here: workspace clippy/unit tests and scenarios (a debug
  workspace build needs ~24 GB; CI runs them on the patched conductor).
- CI: `check` 6 min, shards 45–60 min (all scenarios on the patched conductor). After deploy:
  live `projects.mjs` + `guests.mjs` passed. The owner can confirm in Railway that seed-node
  memory now stays flat. Report upstream (holochain/holochain: genesis eviction;
  wasmerio/wasmer: engine code memory never freed) is still to do; needs a GitHub account the
  owner chooses.

## Done 2026-10-25: guests open project pages (ADR 0052), PR #43 merged (~01:33 UTC / 12:33 pm Melbourne), deployed ~01:51 UTC; live `guests.mjs` and `projects.mjs` passed
STATUS Next item 3. web shows project pages signed out and gates actions with
`requireViewer` → `AuthActionNotice`; ported. **No zome or integrity change** (gateway + UI),
so merging resets nothing.
- Gateway: `POST /public/read` (`routes/public.rs`, `node/public.rs`): the archivist answers an
  allowlist of read-only calls (directory `get_profile`/`get_standing`/`get_listing`/
  `find_listing`; project `get_project`, `list_proposals`, `list_plans`, `list_values`,
  `link_sides`, `list_links`, `list_messages`), msgpack in and out; 403 otherwise, 404 for a
  project the archivist hasn't joined; 20,000 reads per IP per day. New dep `rmpv`.
- UI: `holo/guest.ts`; signed out, `callZome('directory')` and `callCell` on a stand-in project
  cell go to the gateway, so services are unchanged. `AccountGate guests` on `/projects/[id]`.
  web's notice: `features/shell/AuthActionNotice.svelte`, `stores/authNotice.svelte.ts`
  (`requireViewer`), `styles/notice.css` (PORTS). Join/Support on the page, Support on feed
  cards (was a jump to /onboarding) and the chat composer's "Sign in" button use it.
- Bug found and fixed on the way: `holo/connection.ts` `connect()` re-ran `open()` on every call
  while signed out, flipping the state to "connecting" and back, so `AccountGate` remounted the
  page in a loop (thousands of reads, `ERR_INSUFFICIENT_RESOURCES`). Signed out now stays
  signed out until `attach()`.
- Checked in the cloud session: clippy (`gateway`, `-D warnings`), fmt, gateway unit tests (new
  allowlist test); `npm run check`/`lint`, sizes, boundaries; against the local seed node at
  390px: new `tests/e2e/guests.mjs` (can run live: one account), `projects.mjs` (its guest step
  now expects the notice), `chat.mjs`, `messages.mjs`. Not run: scenarios (no zome changed).
- CI: `check` 7 min, shards 36–82 min. After deploy (all three services green 01:51 UTC, ~18 min
  after the merge): `/public/read` refuses an unlisted call with 403 live, and `guests.mjs` and
  `projects.mjs` passed against the live UI on the first run.

## Done 2026-10-24: live calls ring again (ADR 0050 addendum), PR #41 merged (~14:30 UTC / 1:30 am Melbourne 25 Oct), deployed ~15:06 UTC; live `calls.mjs` passed on the first run
The owner can't read Railway logs and asked for a fix that doesn't depend on the seed node.
**Cause, found by watching the live websocket frames** (Playwright `page.on('websocket')`,
frames decoded with `@msgpack/msgpack`): the grant and `send_direct_signal` both succeed and the
`app_direct` signal **does reach Bob's page**, but `services/calls.ts` `directPeer` read the
chat's whole summary before accepting it; on the busy live node that took longer than Ann's
15 s wait, so she saw "Not available right now", hung up, and Bob's late ring closed at once.
One run also lost the first signal while Bob's page reloaded. Not the seed node, the relay or
PR #38's DNA (the same upgrade works locally; agent records on the live bootstrap carry URLs).
- Fix (UI only, no zome change): `findPeer` trusts the seed's creator in a chat someone else
  started (the seed must be in your inbox; direct chats have two people), the call button
  passes the known member (`rememberPeer`), the invite repeats every 2.5 s until `ringing`,
  and a callee ignores invites for calls that already ended.
- **Verified against the live node before merging:** built this UI with
  `VITE_SP_GATEWAYS=https://seed-node-production.up.railway.app`, served it on localhost:4175
  (the gateway accepts it; sign-up needs no passkey) and ran the tests there: the ring check
  3/3 (~3.5 s), full `calls.mjs` passed, `messages.mjs` passed. Also `npm run check`/`lint`,
  sizes, boundaries. **Useful technique for any live-only bug.**
- Still a single point: the bootstrap/relay (`bootstrap-production-f978`, also on Railway)
  that desktop apps use to reach each other (`native/src/network.rs`, one URL). A fallback
  relay list is the next resilience step (STATUS Next item 2).

## Done 2026-10-24: call lines in direct chats (ADR 0051), PR #38 merged (~08:31 UTC / 7:31 pm Melbourne), deployed ~08:48 UTC; live `projects.mjs` and `messages.mjs` passed; live calls didn't ring until the fix above
STATUS.md Next item 1. web's "Voice call · 4:12" / "Call declined" / "No answer" / "Missed call"
lines. **Integrity change (conversation DNA only)**: a public `Call` entry, rules in the new
`crates/sp_call`; directory and project DNA hashes unchanged (`hc dna hash`: `uhC0kX7veWx…`,
`uhC0kJTRl5t…`). The live direct/group test chats made before ~08:48 UTC are on the old network.
- **Open problem: live calls stopped ringing after this deploy.** `calls.mjs` against the live
  UI (3 runs, 08:55–09:06 UTC) fails at the first ring: Ann sees "Not available right now",
  Bob's page gets nothing, no console errors. `messages.mjs` and `projects.mjs` pass live, so
  hosting, the new conversation DNA and the inbox work; only direct signals don't arrive.
  Reproduced the deploy locally and it works: seed node on main's previous hApp (`4e6c1ba`,
  ring OK), restarted on the same data with this hApp (4 old apps removed, as live), new
  accounts ring in ~1 s. Two people on one node need no network to message (both write
  locally), but a direct signal goes out through the node's transport, so the first guess was
  the live node's network/relay state, not the code.
- **A restart didn't fix it**: docs PR #39 merged ~10:29 UTC, seed node redeployed 10:41 UTC;
  at 10:47 UTC live `calls.mjs` and a debug run (3 min wait) failed the same way. Note that
  live calls had passed only once before (PR #36's smoke, 05:13 UTC), so they may never have
  been reliable live. **Needs the owner (Railway access):** `railway logs --service seed-node`
  while running `node tests/e2e/calls.mjs https://ui-production-333c.up.railway.app` (look for
  kitsune2/iroh or "signal" errors; raise `RUST_LOG` to `info,holochain_p2p=debug,kitsune2=debug`
  for one run). Ideas to check in code: whether `send_direct_signal` to an agent on the same
  conductor goes through the relay (`RELAY_URL` = the bootstrap service) and whether the
  production relay accepts it; whether the gateway's `/ws` proxy forwards signals for the new
  inbox cell. Reverting PR #38 would reset the test chats again and probably not help.
- Checked in the cloud session: fmt, clippy (`sp_call`, `messages`, `messages_integrity`,
  `-D warnings`), `sp_call` unit tests, hApp build + DNA hashes, scenarios `calls` (new, 4 min)
  and `messages` (11 min), `npm run check`/`lint`, sizes, boundaries; against the local seed
  node at 390px: `calls.mjs` (now checks "Call declined", one "Voice call · 0:0X" on both
  phones, "No answer"; first run timed out at sign-up on the cold node, rerun passed) and
  `messages.mjs`. "Call declined" is red (computed style checked in the browser). Not run here:
  the other scenario files (CI runs them).
- CI: `check` 5 min, shards 46–81 min (`scenarios (2/4)` took 81 min).

## Done 2026-10-24: voice calls in direct chats (ADR 0050), PR #36 merged (~04:56 UTC / 3:56 pm Melbourne), deployed ~05:13 UTC; live `calls.mjs` and `projects.mjs` passed
STATUS.md Next item 1 ("calls"). web's call screen, ported for direct chats. **UI only, no zome
change**, so merging resets nothing.
- Signalling is Holochain 0.7's **direct signals** (`AppWebsocket.sendDirectSignal` /
  `grantDirectSignalCapability`) on the inbox cell, not a zome: `holo/signals.ts`,
  `services/calls.ts` (accepts a signal only from the other member of one of your direct
  chats). UI: `features/calls/CallHost.svelte` (root layout), `callSession.svelte.ts`,
  `callSignals.ts`, `components/calls/CallControls.svelte`, `services/callMedia.ts`,
  `services/callTones.ts`, `styles/calls.css` (PORTS); the phone button in a direct chat's
  header (`ConversationView.svelte` → `stores/callRequest.svelte.ts`).
- App-only: the callee's device answers an invite with `ringing`; with none in 15 s the caller
  sees "Not available right now" (web's server knew who was online).
- Not ported yet: web's "Voice call · 4:12" / "Missed call" lines in the chat (they need a
  message kind in the conversation DNA: next integrity change).
- Checked in the cloud session: `npm run check`/`lint`, sizes, boundaries; against the local
  seed node at 390px: `calls.mjs` (new: decline, accept with Chromium's fake microphone, timer,
  pill, end, then "Not available right now" with Bob's app closed; passed on the first run,
  twice) and `messages.mjs`. No Rust changed, so fmt/clippy/scenarios weren't run here.
- `tests/e2e/lib.mjs` now launches Chromium with a fake, auto-allowed microphone (all tests).
- CI: `check` 4 min, shards 55–81 min (`scenarios (1/4)` took 81 min again).
- After deploy (all three services green at 05:13 UTC): `calls.mjs` passed against the live UI
  on the first run (ring, decline, accept with audio connecting, timer, end, "Not available
  right now"), so direct signals work between two people hosted on the seed node. Two people
  on different networks may still need a TURN relay for audio (owner-review item 12).

## Done 2026-10-24: photos and files in direct and group chats (ADR 0049), PR #34 merged (~01:40 UTC / 12:40 pm Melbourne), deployed ~02:06 UTC; live `attachments.mjs` and `projects.mjs` passed
STATUS.md Next item 1 ("attachments, then calls"). web's LiveChatPanel attachments, ported for
direct and group chats; project chats stay text-only until the next project-DNA change.
- **Integrity change** (conversation DNA only): public `Chunk` entries (≤1 MB parts) and
  `Message.attachments`. Merging reinstalls hosted apps and leaves the live direct/group chats
  on the old network (test chats), as PR #32 did. **Directory and project DNA hashes are
  unchanged** (`hc dna hash`: directory `uhC0kX7veWx…`, project `uhC0kJTRl5t…`, same as main).
- New crate `crates/sp_attachment` for the rules. Lesson: adding a module to `sp_core` changed
  the directory and project hashes too (every integrity zome links it), so new rules that only
  one DNA needs go in their own crate, and `hc dna hash` before/after any `sp_core` change.
- UI: composer photo/file buttons, previews, errors; photo row, file bubbles; PhotoViewer;
  "Photo"/file name in list previews and quotes (web-backend `_message_preview`).
- Checked in the cloud session: fmt, clippy (`sp_attachment`, `messages`, `messages_integrity`,
  `scenarios`, `-D warnings`), `sp_attachment` unit tests, hApp build + `hc dna hash` (above),
  scenarios `attachments` (new, 6 min) and `messages` (10 min); `npm run check`/`lint`, sizes,
  boundaries; against the local seed node at 390px: `attachments.mjs` (new: photo + 1.5 MB
  file, list "Photo", viewer, download intact; first run timed out at sign-up on the cold node,
  rerun passed), `messages.mjs`, `group-options.mjs`, `chat.mjs`, `mentions.mjs`,
  `public-chats.mjs`. Not run here: the other scenario files (CI runs them).
- CI: `check` 6 min, shards 30–81 min (`scenarios (1/4)` took 81 min; add shards if one nears
  120). Railway: bootstrap and ui ~1 min after the merge, seed-node ~25 min.
- After deploy: `projects.mjs` passed live. `attachments.mjs` run 3 min after the seed node
  restarted failed: Ann's sent photo message never showed in her chat (the send itself
  finished). The rerun ~5 min later passed in full (photo, viewer, 1.4 MB file intact). Same
  cold-node pattern as earlier deploys, but if a sent attachment ever vanishes again, look at
  `post_message` after `upload_chunk` on a freshly reinstalled app.

## Done 2026-10-24: read receipts in direct chats, PR #32 merged (~15:20 UTC 2026-10-23 / 2:20 am Melbourne), deployed ~15:41 UTC; live `messages.mjs` and `projects.mjs` passed
The owner asked for "seen" receipts on 2026-10-23. web shows ✓ (sent) / ✓✓ (read) on your own
messages in **direct chats only** (`peer_read_at`). ADR 0048, `docs/plans/M4-talk.md` "M4c as
built, fourth part".
- **Integrity change** (conversation DNA only): a public `Seen` entry (the private `ReadMark`
  can't be seen by the other person). The hApp version changed: hosted apps reinstalled and
  existing direct/group chats were left on the old network (test chats only); projects and
  people are untouched.
- `mark_read` writes a `Seen` next to the read mark in direct chats; `peer_read_at()` reads the
  other person's newest. UI: `ConversationView` → `withReceipts` → `ChatBubble`.
- Checked in the cloud session: fmt, clippy (`messages`, `messages_integrity`, `scenarios`,
  `-D warnings`), hApp build, scenarios `receipts` (new) and `messages`; `npm run check`/`lint`,
  sizes, boundaries; against the local seed node at 390px: `messages.mjs` (✓ then ✓✓, none in
  the group; first run timed out at sign-up on the cold node, rerun passed), `group-options.mjs`,
  `mentions.mjs`, `chat.mjs`. Not run here: the other scenario files (CI runs them).
- CI: `check` 5 min, shards 27–80 min (`scenarios (0/4)` took 80 min; watch it, CLAUDE.md's
  CI note says add shards if one nears 120). After deploy (~5 min wait for the reinstall):
  `messages.mjs` (✓ then ✓✓) and `projects.mjs` passed against the live UI on the first run.
  The live direct/group chats made before this deploy are on the old network now.

## Done 2026-10-23: links to the app's pages in messages, PR #30 merged (~11:21 UTC / 10:21 pm Melbourne), deployed ~11:33 UTC; live `public-chats.mjs` passed
**UI only, no zome change.** web's `linkifyMessageBody` internal paths and `withoutRawPaths`:
- In every chat bubble, a path to one of the app's pages (`/projects/…`, `/messages…`,
  `/profile/…`, with any `?query`) is a same-tab link; a line "Label: /projects/…" shows as a
  "Label" link. web's other pages (events, threads, posts, …) aren't ported, so their paths stay
  text. Chat-list previews drop raw paths, as web's do. `components/chat/chatFormat.ts`
  (`bodyParts`, `withoutRawPaths`), `ChatBubble.svelte`, `messages/ChatRow.svelte`.
- `tests/e2e/public-chats.mjs`: Ann's reply carries the project's path; her list preview
  drops it and Bob follows the link from the bubble.
- Checked in the cloud session: `npm run check`/`lint`, sizes, boundaries; web's linkify test
  cases by hand (same output); against the local seed node at 390px, all passed:
  `public-chats.mjs`, `mentions.mjs`, `messages.mjs`, `chat.mjs`. No Rust changed.
- CI: `check` 5 min, shards 50–65 min. After deploy: `public-chats.mjs` passed live, first run.

## Done 2026-10-23: project members panel from Messages, PR #29 merged (~09:46 UTC / 8:46 pm Melbourne), deployed ~10:04 UTC; live `public-chats.mjs` passed
**UI only, no zome change**, so merging reset nothing. web's `openProjectMembers`: in a project
chat opened from Messages → Public, the header's "N members" is a button that reads the
project's members and opens the "Project members" sheet (the same `PeopleSheet` as the project
page's member count). `services/publicChats.ts` `projectChatMembers(seed)`;
`PublicChatView.svelte` (its local `state` is now `talk`: a local `state` clashes with the
`$state` rune). `tests/e2e/public-chats.mjs` opens and closes the sheet.
- Checked in the cloud session: `npm run check`/`lint`, sizes, boundaries; `public-chats.mjs`
  against the local seed node at 390px (first run timed out at sign-up on the cold node, the
  rerun passed). CI: `check` 7 min, shards 36–63 min. After deploy (waited for Railway's
  ui/seed-node statuses): `public-chats.mjs` passed against the live UI on the first run.
- Lessons: `scripts/e2e-local.sh` takes test files only (no screens-dir argument; screens land
  in `./screens`, delete them). Running a test against the live UI needs
  `CHROMIUM_PATH=/opt/pw-browsers/chromium` in a cloud session too.

## Done 2026-10-23: scenario exit crash fix, PR #27 merged (~04:38 UTC / 3:38 pm Melbourne)
Test-only (`tests/scenarios`); the owner asked for it as its own PR.
- Cause, measured: after `shutdown()` 17–34 `sqlx-sqlite-worker` threads (the conductors'
  SQLite pools) were still running and took 45–99 s to wind down; a test binary that exited
  meanwhile aborted (`malloc_consolidate`) or segfaulted after every test had passed (PR #25's
  CI on `1df5f05`, main's run #81).
- Fix: `common::shut_down(conductors)` takes the batch by value, drops it, then waits until no
  SQLite worker thread is left (2 min cap, warns if reached). New scenario tests must end with
  it (or `net.shut_down()` / `pair.shut_down()`).
- Cost: shards 36–63 min (were 30–57 on the same base). A single wait at process exit would be
  cheaper but needs `unsafe` (`atexit`), which the workspace forbids.
- The crash was intermittent, so a few green runs don't prove it gone: if a shard dies after
  "test result: ok" again, look for another thread kind outliving the wait (list
  `/proc/self/task/*/comm` after `shut_down`).

## Done 2026-10-23: @mentions in every chat + scroll to the linked message, PR #25 merged (~03:14 UTC / 2:14 pm Melbourne), deployed 2:26 pm; live `projects.mjs` and `mentions.mjs` passed (ADR 0047)
**UI only, no zome change**, so merging reset nothing.
- web's mention menu in every chat composer (project, direct, group; the owner asked for
  direct/group on 2026-10-23), `@name` links in bubbles, and the `mention` notification derived
  on the mentioned person's device: web's project card ("You were mentioned.") or its "Thread"
  card ("<name> mentioned you in <group>."), which opens the chat.
- Suggestions ranked by closeness (owner's ask): people in this chat, then people in your other
  chats, then everyone; exact > prefix > substring, then by name. Follows (and followers of
  followers) slot in when web's follow screens are ported: the app has none yet. Same names
  show their key tag in the menu.
- `?comment=<id>` now centres and marks the message (`.highlighted`); your next message drops
  the mark and the parameter (web's behaviour).
- Also fixed: "Mark all read" pressed before the list loaded forgot cards read one by one; and
  `tests/e2e/notifications.mjs` left the project right after Bob's vote, which could cancel
  the queued vote write (it failed the same way with main's UI); it now waits on the project
  page until the phase moves, as `governance.mjs` does.
- New browser test `tests/e2e/mentions.mjs` (two new accounts, can run live): project chat
  mention, linked message, then a group chat mention and its card.
- Checked in the cloud session: `npm run check`/`lint`, sizes, boundaries; against the local
  seed node at 390px, all passed: `mentions.mjs`, `notifications.mjs`, `governance.mjs`,
  `chat.mjs`, `public-chats.mjs`, `messages.mjs`, `group-options.mjs`. No Rust changed, so
  fmt/clippy/scenarios weren't run here (CI runs them).
- CI: `check` 5 min, shards 29–57 min. One earlier run (`1df5f05`) failed `scenarios (3/4)`
  on the exit crash above, after every test had passed; the next run was green.
- Lessons: after many runs the local conductor answered `database is locked` (SQLite) on every
  project creation; restarting `scripts/cloud-seed.sh` cleared it. The restart starts with an
  empty data dir, so the founders' first phrase sign-in waits on a restore (ADR 0035) and a
  retry within a few minutes fails with "already marked as restoring"; wait, then rerun.

## Done 2026-10-21: Messages unread badge, PR #19 merged (~07:27 UTC / 6:27 pm Melbourne), deployed 2026-10-22 ~22:43 UTC after Railway's outage; live `messages.mjs` passed
- PR #18 (scenario tests await conductor shutdown) merged ~05:46 UTC; PR #17 (docs) ~06:19 UTC.
- web's `unreadCounts.messages` on the send icon (top bar and phone bottom nav), as on web:
  unread messages in personal chats you haven't deleted plus your project chats (Public tab).
  `services/unread.ts` counts it; the shell refreshes it with the bell every 30 s; the Messages
  page sets it from its own lists, so reading a chat clears it at once (`stores/messageBadge`).
- Project-chat counts use this device's read marks (ADR 0044), so another device can still
  show them as unread. Muted chats still count, as on web.
- `tests/e2e/messages.mjs` now checks that the count appears and clears; it passed against the
  local seed node (cloud session), with fast checks (svelte-check, lint, boundaries, sizes).

## Done 2026-10-21: M4d notifications, first part, PR #16 merged (~04:04 UTC / 3:04 pm Melbourne), deployed
web's Notifications page and the bell's unread count (ADR 0046, `docs/plans/M4-talk.md`
"M4d as built"). **No integrity change** (one new directory coordinator call,
`find_listing(seed)`), so merging resets nothing.
- Derived on each device from the projects opened on its node (like the Public tab): web's
  `prj-phase-vote` ("Vote on advancing to …"), `prj-phase-done` ("Project phase change
  executed") and `reply` on project chats ("Replied to your comment" / "… your project"), with
  web-backend's recipients. Read marks in browser storage per person; the first look on a
  device counts older activity as read. The bell (top bar + bottom nav) refreshes every 30 s.
- Cards open the project: chat replies `?tab=chat&comment=…`, votes `?open=vote&voteTarget=…`
  (opens the Vote sheet on that proposal, as web's link does).
- Left out on purpose: web's `prj-plan-lead` (off by default on web; no settings here yet),
  every kind without an app feature (follows, shares, invites, roles, mentions).
- Checked in the cloud session: fmt, clippy (`identity`, `scenarios`, `-D warnings`), hApp
  build, scenario `projects` (with `find_listing`), `npm run check`/`lint`, sizes, boundaries;
  against the local seed node at 390px: `notifications.mjs` (new; its first run timed out at
  Ann's project creation on a cold node, the rerun passed in full), `governance.mjs`,
  `chat.mjs`, `projects.mjs`, `public-chats.mjs`. Not run here: the other scenario files and
  the full workspace clippy/unit tests (no other Rust changed; CI runs them).
- `notifications.mjs` needs the dev founders (a phase vote must pass), so it runs locally only.
- CI: `check` 5 min, shards 27–28 min. After deploy (waited ~8 min): `/readyz` ok,
  `projects.mjs` and `public-chats.mjs` passed against the live UI on the first run.
- Lessons: the home index's cards (`/index/projects`) already carry `networkSeed`, a cheaper
  seed→listing source for later. `pkill -f gateway` kills your own shell when the command line
  contains "gateway"; stop the local seed node by its task instead.

## Done 2026-10-21: M4c group options, edits and pins, PR #14 merged (~00:17 UTC / 11:17 am Melbourne), deployed
web's chat menu, Chat info sheet, message edits and pins (ADR 0045, `docs/plans/M4-talk.md`
"M4c as built, third part"). CI: `check` 5 min, shards 38–65 min. After deploy:
`group-options.mjs` and `projects.mjs` both failed on the first run right after the seed node
restarted (this deploy reinstalls every hosted app), then both passed in full on the rerun 5 min
later. After a hApp-version deploy, wait ~5 min before smoking.
- **Integrity change** (conversation DNA only): hApp version changes, hosted apps reinstall;
  existing direct/group chats are left on the old network (test chats only). Projects and
  people are untouched.
- Groups: the manager (the creator, until they leave and hand over to the longest-standing
  member) renames, adds and removes; anyone leaves ("… left the group." line). Writes cite
  their authority (`by`: `None` for the creator, else the handover `Leave`); who counts when is
  replayed by `sp_core::roster`. Edits are `Edit` entries (newest shows, "edited"); pins are
  `Pin` entries (either person in a direct chat, the manager in a group, at most 5); pin, mute,
  delete are a private `Prefs` entry (project chats: browser storage).
- UI: row right-click / long press and the chat header open web's chat menu (View profile,
  Pin, Mute, Chat info, Delete, Leave group); Chat info sheet; pin bar; Edit and Pin in the
  message menu with the composer's edit mode.
- Checked in the cloud session: fmt, clippy (workspace, `-D warnings`), unit tests, `npm run
  check`/`lint`, sizes, boundaries, hApp build, scenarios `groups` (new, 2 tests) and
  `messages`; against the local seed node at 390px: `group-options.mjs` (new), `messages.mjs`,
  `public-chats.mjs`, `chat.mjs`. Not run here: the other scenario files (CI runs them),
  `projects.mjs`, `governance.mjs`, `links.mjs` (no project code changed).
- Lessons: web's `onTogglePin(id, pinned)` passes the message's *current* state (toggle), the
  zome takes the wanted one; `ChatState.pin` flips it. `scripts/dev-seed.sh` only builds the
  hApp when none exists: after a zome change run `bash scripts/build-happ.sh` before restarting
  the seed node, or it serves the old coordinator. Don't name an extern `rename` (libc).

## Done 2026-10-20: M4c Public tab, PR #12 merged (~14:48 UTC), deployed
web's Messages → Public tab (ADR 0044, `docs/plans/M4-talk.md` "M4c as built, second part").
**UI only, no zome or integrity change**, so merging resets nothing.
- Personal/Public tabs with unread badges; Public lists the chats of projects you're a member of
  or wrote in, from the project clones on this node (read by seed, not via directory listings),
  with web's "Project" label, last message and unread pill. A row opens the project chat inside
  Messages (`?project=<seed>`, header "Project chat · N members").
- Read marks for project chats live in `localStorage` per person + project seed (no private
  entries in the project DNA yet); reading on Messages or the project's Chat tab clears them.
- Checked in the cloud session: `npm run check`/`lint`, sizes, boundaries; against the local
  seed node at 390px: new `tests/e2e/public-chats.mjs` (two new accounts, can run live),
  `messages.mjs`, `chat.mjs`. No Rust changed, so fmt/clippy/scenarios weren't rerun here (CI
  runs them).
- Lesson: an e2e test that leaves the page while a chat copy still says "Sending…" cancels the
  write (the websocket goes with the page); wait for "Sending…" to go first.
- CI: `check` 5 min, shards 32–43 min. After deploy: `public-chats.mjs` passed against the live
  UI. `projects.mjs` failed once right after the seed-node restart (Bob's own "2 members" took
  over 60 s) and passed on the rerun: the same cold-node slowness seen after earlier deploys.

## Done 2026-10-20: M4c direct and group chats (first part), PR #10 merged (~08:04 UTC), deployed
web's Messages page, first part (ADR 0043, `docs/plans/M4-talk.md` "M4c as built"):
- Conversation DNA gets the `messages` zome pair. Each chat is a clone seeded
  `<creator>.<direct|group>.<nonce>`; the creator invites, messages cite their author's
  invitation, group names are the creator's `Title`s, read marks are private entries.
  Invitations reach people through the base conversation cell (the inbox): an `Inbox` link on
  their key whose tag is the seed encrypted to them.
- UI: `/messages` (Personal list with unread counts, the New message sheet for direct/group,
  a conversation with web's header + the shared chat panel), and the profile's Message button.
  The project chat's state moved to `components/chat/chatState.svelte.ts` (a `ChatSource`).
- **Integrity change** (conversation DNA only): the hApp version changes, so hosted apps are
  reinstalled; directory and project DNAs are unchanged.
- Not yet: the Public tab (project chats in the list), group options (rename, members, leave),
  the chat row menu, edits, pins, attachments, calls (see ADR 0043 Consequences).
- Also fixed: `tests/e2e/chat.mjs` counted the Take part tab's hidden dock as "a dock over
  the chat" (stale since PR #3); it now counts visible docks.
- Checked in the cloud session: fmt, clippy (workspace, `-D warnings`), unit tests, `npm run
  check`/`lint`, sizes, boundaries, hApp build, scenario `messages` (2 tests, ~10 min), and
  against the local seed node at 390px: `tests/e2e/messages.mjs` (profile Message → DM, unread
  pill, reply, group from New message), `chat.mjs`, `projects.mjs`, `links.mjs`. Compared with
  web's MessagesPage/ComposeMessageSheet markup and styles (live web's Messages needs an
  account, so no live 390px comparison). Not run here: the other scenario files (CI runs them),
  `governance.mjs`.
- Lessons: a chat's read mark and a message sent at once race on the source chain, so every
  chat write on the Messages page goes through `features/messages/writes.ts` `inTurn`. An
  `$effect` that calls `state.start()` must `untrack` it, or the first refresh re-runs the
  effect, whose cleanup stops the poll.
- CI: `check` 7 min, shards 21–63 min. After deploy: `projects.mjs` and `messages.mjs` passed
  against the live UI. The first `messages.mjs` run right after the seed node restarted timed out
  at Ann's first send (the first chat clone on a cold node); the rerun passed in full. Watch for
  slow first sends after deploys.

## Done 2026-10-20: M3e links between projects, PR #8 merged (~03:56 UTC), deployed
web's Links tab (ADR 0042, `docs/plans/M3-governance.md` "M3e"), ported from web's
`features/detail-links/*` and web-backend `detail_links.py`:
- A link request is two `Link` proposals, one in each project: the proposer files it in their
  project, the app mirrors it into the other (the mirror may come from a non-member, so
  `Proposal.membership` is now optional). Each project votes on its half under its own rules;
  the link is active once both pass. Severing works the same way, from either side.
- `governance` zome: `propose` (link halves), `link_sides`, `list_links(others)`; pairing rules in
  `sp_governance::links`. UI: `features/projects/LinksTab.svelte`, `components/links/*`,
  `services/links.ts`, `styles/links.css` (PORTS).
- **Integrity change**: merging resets projects again (people stay).
- Also fixed: two reads of a project not opened before (the Links tab reads a linked project's
  page and its link halves together) cloned it twice and failed with `DnaDefMissing`;
  `holo/cells.ts` now shares one lookup per seed.
- Checked in the cloud session: fmt, clippy (workspace, `-D warnings`), unit tests, `npm run
  check`/`lint`, sizes, boundaries, hApp build, scenario `links` (6 min), and against the local
  seed node at 390px: `tests/e2e/links.mjs` (propose → both approve → sever), `projects.mjs`,
  `governance.mjs`. Compared with live web's Links tab at 390px: same layout. Not run here: the
  other scenario files (CI runs them) and `chat.mjs`.
- CI: `check` 6 min, shards 23–45 min. After deploy: projects reset (index 0), `projects.mjs`
  passed against the live UI, and the live Links tab at 390px shows web's sections (Active
  links, Pending link requests with "Propose a new link", Historical links). `links.mjs` needs
  the dev founders, so it only runs locally.

## Done 2026-10-20: PR #6 merged (~00:48 UTC), deployed, live smoke passed
**M3d project values** (ADR 0041, `docs/plans/M3-governance.md` "M3d"), ported from web's
ProductiveLifecyclePhaseOne / ProjectValueCard / DiscreteScale / AddValueSheet:
- In the Proposal phase members add values ("What should this achieve?") and rate them 1–10;
  duplicates (case/spacing ignored) are refused; only Full members' latest votes count. Values
  averaging ≥5 become plan criteria ("Shared value" rows in the plan sheet), and a plan's score
  now averages every rated criterion, as web's does.
- **Integrity change** (new `Value`/`Importance` entries, two link types, value criteria in
  `Rating`): merging resets projects again (people stay).
- **CI split**: `scripts/check-fast.sh` (everything but scenarios) + `scripts/check-scenarios.sh
  [<shard> <shards>]`; CI runs the scenarios as 8 parallel shards (`ci.yml` job `scenarios`).
  `check-all.sh` still runs everything. New scenario files are picked up automatically.
- `scripts/port-styles.py` now dedents web's `<style>` blocks. Regenerating everything today
  also pulls ~500 lines of web's newer styles into the other stylesheets; that drift was left
  out of this PR (review it separately: run the script and read the diff).
- New `scripts/cloud-scenarios.sh`: scenarios in a cloud container (no-IPv6 shim, no debug info).
- Verified in the cloud session before pushing: `cargo fmt --check`, `cargo clippy --workspace
  --all-targets -D warnings`, `cargo test --workspace --exclude scenarios`, `npm run check`/`lint`,
  sizes, boundaries, hApp build; scenarios `values` and `plans` (scoring changed); the whole of
  `tests/e2e/governance.mjs` at 390px against the local seed node (now with the values step:
  add, both founders rate 8/10, the plan sheet shows it as "Shared value"). Compared with live
  web's Proposal tab at 390px: same layout. Not run here: the other scenario files (CI runs them)
  and `projects.mjs`/`chat.mjs`.
- First sharded CI run: `check` 6 min, shards 18–55 min, so ~55 min wall clock (was ~130).
- After deploy: projects reset (index 0), `projects.mjs` passed against the live UI, and the live
  Take part tab at 390px shows "No values yet", "+ Add value" and "Phase ▾" in the bar.

## Done on 2026-10-19 (earlier cloud session)
All merged and deployed; the live smoke (`tests/e2e/projects.mjs` against the live UI) passes.
- **PR #1** M3b plans + M4a project chat. Its CI first hit the 60-minute job timeout while the
  scenarios were still passing; the timeout is now 150 min (`ci.yml`). M3c's run took 132 min,
  so the next scenario file will need the timeout raised again or the scenarios split into
  parallel jobs (a session's safety check blocked adding a "skip scenarios" switch to do that;
  the owner may want to make that change).
- **PR #3** Take part tab ported from web: stepper → "Current phase" header → plans → Phase ▾
  menu; `PhaseWorkToolbar` packs "Add plan" + "Phase ▾" into the tab's own action bar (Join,
  Vote for phase changes); the Details bar keeps Edit and edit/update votes. Plus README/STATUS,
  cloud scripts, and a fixed race in `projects.mjs`.
- **PR #4** M3c (ADR 0040): phase returns (web's return sheet, from the Phase menu's "Return to
  an earlier phase"), the challenge window (rules only), and web's required "why" on every phase
  change. Integrity change: projects were reset again at ~19:35 UTC (people stay).
- Verified locally before merging: fmt, clippy, unit tests, hApp build, scenarios `reverts`,
  `challenges`, `governance`; `tests/e2e/governance.mjs` end to end at 390px (edit, update,
  advance with a reason, plan rated and approved, return voted through) and `projects.mjs`.

## For the owner to review (choices made without you)
1. **Challenge list removed.** M3c's agent built an app-only "Counts you can challenge" list;
   web has no such screen, so it was removed. The challenge rules stay (zome, `sp_governance`,
   scenarios). Restore `ui/src/lib/components/governance/ChallengeList.svelte` and its
   `additions.css` block from commit `a0ce970` if you want it. Until then challenges can only be
   made through the zome.
2. **"Why" on every phase change.** web asks for a reason when advancing or closing; this needed
   `ProposalKind::Phase.reason` (integrity), so it went into M3c's reset.
3. **CI timeout 150 min** (see above).
4. `claude/m4b-threads` untouched and parked, as decided.
5. **DM privacy trade-offs (ADR 0043).** A chat is private because its seed is secret and
   validation stops uninvited writers; there is no join-time membrane proof yet. The inbox
   reveals *who invited whom* (not which chat or what was said). Gateway-hosted people's chats
   are readable by that node's operator, as on web today. Say if any of these must change
   before DMs go further.
6. **Public tab and notification read marks are per device** (ADR 0044, 0046): project chats have no private read
   marks in the DNA yet, so "unread" is kept in the browser until an integrity change adds them.
   The members count in a project chat's header opens the members sheet, as on web.
7. **Group "Delete" leaves the group** (ADR 0045), with the "left the group" line and a
   handover; web's Delete drops the membership silently and can leave a group without its
   manager. Removed people can still read what the chat gossips (they know its seed).

8. **Notifications leave out "Plan became leading"** (ADR 0046): web sends it only to people
   who turn it on in settings, and the app has no notification settings yet.

9. **Mentions with shared names** (ADR 0047): app usernames aren't unique, so `@name` links to
   the nearest person with that name (this chat, then your other chats, then the directory),
   the menu shows key tags when names repeat, and everyone with that name is notified.

10. **Read receipts reset the live test chats** (ADR 0048, 2026-10-23): the new public `Seen`
    entry was an integrity change, so direct/group chats made before ~15:41 UTC are on the old
    network. There is no setting to turn receipts off (web has none either).

11. **Attachments reset the live test chats again** (ADR 0049, 2026-10-24): chats made before
    ~02:06 UTC are on the old network. Files are 1 MB public `Chunk` entries readable by anyone
    with the chat's seed (and by a gateway's operator for hosted people, as on web's server);
    deleting a message marks its parts deleted, but peers may keep the bytes. Project chats get
    attachments with the next project-DNA reset; say if you want that sooner.

12. **Calls (ADR 0050)**: signalling uses Holochain direct signals on the inbox network; any
    node there may signal you, and the UI drops all but your direct chats' other members.
    "Not available right now" comes after 15 s without the callee's device answering (web's
    server knew at once). Only STUN is configured (as on web), so some networks won't connect
    audio until a TURN relay is set (`VITE_CALL_TURN_URLS`, `_USERNAME`, `_CREDENTIAL` on the
    ui service). Call lines came with ADR 0051 (2026-10-24), which reset the live test chats
    again; when the other person is busy the caller's line says "Call declined" (web: missed).

13. **Right rail lists personal services' activities** (2026-10-27): web-backend's rail only
    reads projects in `phase-5`, so on web a personal service's activities (its Activity phase
    is `phase-1`) never reach the rail. The app lists them; say if it should copy web exactly.

14. **New members' posts and threads count against the 2-listings-per-7-days cap** (ADR 0070):
    the cap lives in directory integrity and counts every listing (projects, events, help
    requests, channels, threads, posts). web lets anyone who can participate post freely.
    Lifting it for posts and threads needs a directory integrity change (an account reset).
    Say if that is worth it now.

## Live (test network, Railway project `social-production-p2p`)
- UI https://ui-production-333c.up.railway.app · seed node https://seed-node-production.up.railway.app (`/readyz`, `/index/projects`).
- Deployed `main`: `419e2c8` (PR #61, merged 2026-10-27 16:19 UTC; seed node live 16:39 UTC; directory network reset). `/readyz` now shows `uptimeSecs` and `rssMb`. Merging to `main` redeploys bootstrap, seed-node and ui. **Check that the UI actually changed before smoking:** `curl https://api.github.com/repos/social-production/app/commits/<sha>/status` lists Railway's per-service deploy statuses. A red ✗ on a merge commit is usually its `main` CI run cancelled by the next merge, not a failed deploy. **Lessons from Railway's 2026-10-21/22 outage and security incident:**
  - Merges made while builds were paused were never deployed, and pushes after recovery didn't trigger deploys either.
  - Railway's Redeploy rebuilds the same old commit. To pick up the latest `main`, the owner switched the services' deploy branch away and back (Settings → Source), or reconnected the repo; on desktop, Cmd/Ctrl+K → "Deploy latest commit" also works.
  - After the cache wipe the seed node took ~40 min to build (normally ~5).
  - Check status.railway.com first when deploys stall. A Railway project token as `RAILWAY_TOKEN` in the cloud environment would let a session read deployments (`backboard.railway.com` is reachable).
- Live: M1 identity, M2 projects, M3a governance (edit/update/phase proposals, Vote sheet), M3b
  plans, M3c phase returns (+ challenge rules), M3d project values, M3e links between projects (web's Links tab), M4a project chat, M4c Messages (direct and group chats with read receipts, photos and files, voice calls in direct chats with call lines, Public tab with the project members panel, group options, chat menu, edits and pins), M4d notifications (phase votes, phase changes, chat replies, @mentions; bell count), @mentions and links to the app's pages in every chat, web's Take part tab, M5b activities and roles, the activity button's right rail, guests browse the feed and open project pages signed out, usernames without spaces. Founders
  thebestdamn + BigBird.

## Cloud environment notes
- Nix is preinstalled. `bash scripts/cloud-nix.sh <cmd>` runs a command in the dev shell: the
  proxy blocks GitHub tarball downloads for other repos, so it fetches the flake inputs over git
  at their locked revisions. holochain-ci.cachix.org must be in the environment's allowed
  domains (the owner added it on 2026-10-19).
- `bash scripts/cloud-seed.sh` starts the local seed node: the container has no IPv6 and
  Holochain's interfaces refuse to start without it, so it preloads `scripts/cloud/noipv6.c`.
  Browser tests: `CHROMIUM_PATH=/opt/pw-browsers/chromium bash scripts/e2e-local.sh tests/e2e/<t>.mjs`
  (first sign-in after start can take >2 min; rerun if it times out).
- The live web site (web-production-885ec) is reachable for read-only 390px comparisons.

## Known gaps
- **Railway security incident (2026-10-21, status.railway.com, blog.railway.com/p/security-disclosure-wednesday-october-21st):** an attacker may have read decrypted environment variables, and Railway advises rolling them all. Builds are back (2026-10-22), so rolled values can now be deployed; **not done yet** (owner). **Do not change `LAIR_PASSPHRASE` in Railway on its own:** it decrypts the keystore on `/data`, so a new value without re-encrypting the keystore locks every hosted agent's key away for good. The passphrase alone is useless without the keystore file, and the disclosure is about variables, not volumes. Re-keying needs a planned step (an ADR). The P2P project's other variables (`BOOTSTRAP_URL`, `RP_ID`/`RP_ORIGIN`, `POW_BITS`, …) are not secrets. The live FastAPI project (`social-production`) holds real secrets: `JWT_SECRET` (rolling it signs everyone out), the Postgres and Redis passwords, `GITHUB_TOKEN`, `TURNSTILE_SECRET_KEY` and `GEOCODING_PROVIDER_API_KEY`. Roll those. Its `MESSAGE_ENCRYPTION_KEY` decrypts every stored DM, so it needs MultiFernet re-encryption, not a plain swap.
- **Seed-node memory:** the 3.2 GB (2026-10-21) and the out-of-memory crash (2026-10-25) were a Holochain/wasmer module leak, ~350 MB per new account; fixed by ADR 0053 (deployed 2026-10-25 ~05:25 UTC). After it deploys, live test runs that create accounts are cheap again.
- Plan rating/vote errors go to `detail.error`, which is only shown when the project fails to
  load, so a refused write is invisible. The optimistic plan sheet doesn't roll back, and leaving
  the page before queued writes finish cancels them.
- Guests open project pages read-only (ADR 0052); @names in chat stay plain text for them. Project chats have no photos or files yet (ADR 0049).
- Challenges have no screen (see "For the owner to review").

## Next (roadmap order; set by the cloud session on 2026-10-25, see Owner direction)
Port each screen from web (Owner decisions); compare at 390px with live web first.
1. **Parity pass** (checklist: `docs/plans/PARITY.md`): what's left there waits on features
   (follows, scopes, places, moderation); pick items up as their features land.
2. **M5 Work** (plan: `docs/plans/M5-work.md`): M5a follows + feedback (done, ADR 0054; its
   mention ranking and Home scope in PR #54), M5b activities and roles (done; the rail's link
   and plan votes done), **M5c** events (first part in flight, ADR 0057) and help requests (event cards in the feed;
   project-activity cards wait on Region, M6), M5d communities and channels. Each starts with
   its ADR.
3. Resilience (ADR 0036/0037): a fallback bootstrap/relay list for devices (today one Railway
   URL); archivist + index into a crate any full node runs; desktop Storage setting.
4. CI: scenarios run in 8 shards, balanced by each file's measured minutes
   (`scripts/check-scenarios.sh`, 2026-10-30); add shards or update the minutes if one nears
   120 min.

## How to work (things learned the hard way)
- **Commands:** run them inside nix, `nix develop --accept-flake-config --command bash -c '…'` (PATH may need `/nix/var/nix/profiles/default/bin`). The path has a space; quote it.
- **Fast checks:** `bash scripts/check-boundaries.sh`, `bash scripts/check-sizes.sh`, `npm run check -w ui`, `npm run lint -w ui`, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --exclude scenarios`. Look for `error: test failed`, not just `FAILED`.
- **Scenarios:** `bash scripts/build-happ.sh`, then `bash scripts/check-scenarios.sh` (slow: ~10–15 min per file; `cargo test -p scenarios --test <file> -- --test-threads=1` for one). In a cloud session use `bash scripts/cloud-scenarios.sh`: without its no-IPv6 shim every conductor fails with "Address family not supported by protocol".
  - **Disk (2026-10-31):** each scenario run leaves its conductors' databases in
    `/tmp/nix-shell.*/holochain-test-environments*` (365 folders, ~8 GB, found this session).
    With nothing running, `rm -rf /tmp/nix-shell.*` frees them (needs the owner's OK in auto
    mode); `rm -rf target/debug/incremental` frees ~1 GB more. Check `du -sh /tmp` first.
  - **Disk:** a debug build of the whole workspace is ~24 GB and fills a cloud session's disk. Build with `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0` (the cloud scripts do), and never mix the stock `cargo` with the Nix one in one `target/` (two full builds); `rm -rf target/debug` if `df` shows the disk filling.
  - Settle each DNA's cells separately with `await_consistency`.
  - Never depend on two integrity crates in one test binary; mirror types with serde instead.
  - **End every test with `common::shut_down(conductors).await`** (or `net.shut_down().await` / `pair.shut_down().await`): it shuts the conductors down, drops them and waits for their SQLite worker threads to exit (PR #27). Otherwise the binary can crash at exit after every test has passed: SIGSEGV after `plans` (PR #13), `malloc_consolidate(): unaligned fastbin chunk` after `groups` (PR #17) and `values` (PR #25).
- **Browser tests:**
  - Start the local seed node: `RP_ORIGIN=http://localhost:4174 bash scripts/dev-seed.sh`. It builds a dev hApp whose founders are `tests/e2e/fixtures/dev-founders.json`, so governance can be tested.
  - Wait for "zomes compiled" in its log.
  - Run `bash scripts/e2e-local.sh tests/e2e/{accounts,projects,chat,governance}.mjs`. Signed out, the home page is the feed; sign-up is at `/onboarding`.
- **Live smoke:** `curl` `/readyz` and `/index/projects`, then any browser test file against `https://ui-production-333c.up.railway.app` (`CHROMIUM_PATH=/opt/pw-browsers/chromium node tests/e2e/<t>.mjs <url>`). Since PR #61 the dev founders are founders on the live test network too, so every file runs live (allowed on this test network: they create test accounts). Never create accounts on the FastAPI site; the live web site may be browsed read-only to compare layouts.
- **Styles:** never hand-edit `ui/src/lib/components/styles/*.css`. Add rows to `styles/PORTS` and run `python3 scripts/port-styles.py`. App-only CSS goes in `styles/additions.css` (ADR 0032).
- **One write at a time per person:** two zome calls that write to the same source chain at once are refused (the chain head moved). Optimistic UI that fires several writes (ratings, votes) must queue them, as `DetailState.inTurn` does.
- **A write isn't in your own network reads at once:** after a vote, `tally.my_vote` can still be
  empty for a while; UI flows keep what they just wrote (as `LinksState.cast` does).
- **Killing a process by name kills your shell** when your own command line contains the
  pattern (also a `/proc/*/cmdline` loop). Start servers with `& echo $! > pidfile` and kill
  that pid; `ss` isn't installed in the cloud container.
- **`tests/e2e/shell.mjs`:** twice (not reproduced since, 6 runs) the very first More click found
  no rendered app right after `e2e-local.sh` started its preview; the test now prints the URL,
  the page text and `more-fail.png` when that happens. Look there before calling it a flake.
- **Memory:** measure on a release build (`RELEASE=1 bash scripts/dev-seed.sh`); a debug build
  is no smaller, but production is release. The jemalloc heap-profile recipe is in ADR 0053's
  STATUS entry.
- **Debugging the UI:** errors caught into `detail.error` are not shown once the project has loaded; log them temporarily, or find them with a Playwright script that listens to `console`. When stopping a preview server, kill it by port (`ss -ltnp | grep :4174`). `pkill -f "<its command>"` also matches your own shell and kills it.
- **`sp_core` changes change every DNA's hash** (2026-10-24): even an unused new module did. Rules for one DNA go in their own `crates/sp_*` crate; compare `hc dna hash dnas/<d>/sp_<d>.dna` against main before pushing any shared-crate change.
- **hdk externs** are unmangled C symbols: never name one after a libc function (`close`, `read`, `write`, `send`…), or native test binaries crash.
- **Keystore:** lair is vendored with a 64 KiB message limit (ADR 0034). Keep one lair entry per person.
- **Seed-node logs:** `railway logs --service seed-node`. `RUST_LOG` is back to `info,holochain=warn,kitsune2=warn`. Never print `LAIR_PASSPHRASE`.
- **Owner preferences:**
  - UI is ported from `web/`, not designed;
  - check at 390px first;
  - no "Edit profile" button;
  - Grokbot is paused until ~2026-10-25, so Claude tests;
  - keep `README.md` Status current after each deploy;
  - commit messages are one plain sentence plus the Co-Authored-By line.
