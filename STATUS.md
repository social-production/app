# Status: where the P2P app stands (updated 2026-10-26 by a cloud session; see "In flight" and the newest "Done")

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

## In flight 2026-10-26: @mention suggestions ranked by follows (ADR 0047 update), branch `claude/social-production-handoff-dsam61`
STATUS Next (M5a leftovers). **UI only, no zome change**, so merging resets nothing.
- web-backend's `people_suggestions.py` order: people you follow, then your followers
  (accepted only). The app's circles are now: this chat, following, followers, your other
  chats' members, everyone (`domain/mentions.ts` `MentionPool`, `services/follows.ts`
  `followCircles`, `services/messages.ts` `mentionPool`).
- Test: `tests/e2e/follows.mjs` now has Ann follow Cat and checks her project chat's menu for
  "@<stamp>" lists Cat before Bob (a follower whose name sorts first).
- Checked: `npm run check`/`lint`, sizes, boundaries; against a local release seed node
  (`RELEASE=1 bash scripts/cloud-seed.sh`): `follows.mjs`, `mentions.mjs`.
- Live at start of session (~13:00 UTC 2026-10-26): `/readyz` uptime 15.4 h (no restart since
  the M5a deploy), `rssMb` 629.
- **Next after this:** web's Home scope (decided 2026-10-26): the feed toolbar's scope menu
  (Home / Global; Region waits on M6 places) and the audience menu (Following only /
  Following + popular), with web's `combineFeeds` default (on; the app has no setting for it).
  Home = items from joined channels/communities (none until M5d) merged with the personal feed
  (`mergeHomeFeed`); the app's personal items are the projects of people you follow, shown with
  web's `PersonalActivityCard` (+ `PersonalFeedIdentity`), which the profile feed should use
  too (PARITY). Gateway: `/index/projects?creator=` takes a comma-separated list. "Popular"
  adds discovery posts/threads on web: none exist in the app yet.

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
- Known, not from this branch: the UI build warns that `detail.css` has a multi-line
  `:global(` the port script didn't unwrap (`.project-overview .signal-row`), so those rules
  are dropped; fix in `scripts/port-styles.py`'s `:global` regex.
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
  [<shard> <shards>]`; CI runs the scenarios as 4 parallel shards (`ci.yml` job `scenarios`).
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

## Live (test network, Railway project `social-production-p2p`)
- UI https://ui-production-333c.up.railway.app · seed node https://seed-node-production.up.railway.app (`/readyz`, `/index/projects`).
- Deployed `main`: `f55b5e9` (PR #50, deployed 2026-10-25 ~14:46 UTC, ~29 min after the merge; gateway). `/readyz` now shows `uptimeSecs` and `rssMb`. Merging to `main` redeploys bootstrap, seed-node and ui. **Check that the UI actually changed before smoking:** `curl https://api.github.com/repos/social-production/app/commits/<sha>/status` lists Railway's per-service deploy statuses. A red ✗ on a merge commit is usually its `main` CI run cancelled by the next merge, not a failed deploy. **Lessons from Railway's 2026-10-21/22 outage and security incident:**
  - Merges made while builds were paused were never deployed, and pushes after recovery didn't trigger deploys either.
  - Railway's Redeploy rebuilds the same old commit. To pick up the latest `main`, the owner switched the services' deploy branch away and back (Settings → Source), or reconnected the repo; on desktop, Cmd/Ctrl+K → "Deploy latest commit" also works.
  - After the cache wipe the seed node took ~40 min to build (normally ~5).
  - Check status.railway.com first when deploys stall. A Railway project token as `RAILWAY_TOKEN` in the cloud environment would let a session read deployments (`backboard.railway.com` is reachable).
- Live: M1 identity, M2 projects, M3a governance (edit/update/phase proposals, Vote sheet), M3b
  plans, M3c phase returns (+ challenge rules), M3d project values, M3e links between projects (web's Links tab), M4a project chat, M4c Messages (direct and group chats with read receipts, photos and files, voice calls in direct chats with call lines, Public tab with the project members panel, group options, chat menu, edits and pins), M4d notifications (phase votes, phase changes, chat replies, @mentions; bell count), @mentions and links to the app's pages in every chat, web's Take part tab, guests browse the feed and open project pages signed out, usernames without spaces. Founders
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
2. **M5 Work** (plan: `docs/plans/M5-work.md`): M5a follows + feedback (one directory reset;
   ADR 0054 drafted as **proposed**, build it next),
   M5b activities and roles + the waiting project-chat items (one project reset), M5c events
   and help requests, M5d communities and channels. Each starts with its ADR.
3. Resilience (ADR 0036/0037): a fallback bootstrap/relay list for devices (today one Railway
   URL); archivist + index into a crate any full node runs; desktop Storage setting.
4. CI: scenarios run in 4 shards (~80 min worst); add shards if one nears 120 min.

## How to work (things learned the hard way)
- **Commands:** run them inside nix, `nix develop --accept-flake-config --command bash -c '…'` (PATH may need `/nix/var/nix/profiles/default/bin`). The path has a space; quote it.
- **Fast checks:** `bash scripts/check-boundaries.sh`, `bash scripts/check-sizes.sh`, `npm run check -w ui`, `npm run lint -w ui`, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --exclude scenarios`. Look for `error: test failed`, not just `FAILED`.
- **Scenarios:** `bash scripts/build-happ.sh`, then `bash scripts/check-scenarios.sh` (slow: ~10–15 min per file; `cargo test -p scenarios --test <file> -- --test-threads=1` for one). In a cloud session use `bash scripts/cloud-scenarios.sh`: without its no-IPv6 shim every conductor fails with "Address family not supported by protocol".
  - **Disk:** a debug build of the whole workspace is ~24 GB and fills a cloud session's disk. Build with `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0` (the cloud scripts do), and never mix the stock `cargo` with the Nix one in one `target/` (two full builds); `rm -rf target/debug` if `df` shows the disk filling.
  - Settle each DNA's cells separately with `await_consistency`.
  - Never depend on two integrity crates in one test binary; mirror types with serde instead.
  - **End every test with `common::shut_down(conductors).await`** (or `net.shut_down().await` / `pair.shut_down().await`): it shuts the conductors down, drops them and waits for their SQLite worker threads to exit (PR #27). Otherwise the binary can crash at exit after every test has passed: SIGSEGV after `plans` (PR #13), `malloc_consolidate(): unaligned fastbin chunk` after `groups` (PR #17) and `values` (PR #25).
- **Browser tests:**
  - Start the local seed node: `RP_ORIGIN=http://localhost:4174 bash scripts/dev-seed.sh`. It builds a dev hApp whose founders are `tests/e2e/fixtures/dev-founders.json`, so governance can be tested.
  - Wait for "zomes compiled" in its log.
  - Run `bash scripts/e2e-local.sh tests/e2e/{accounts,projects,chat,governance}.mjs`. Signed out, the home page is the feed; sign-up is at `/onboarding`.
- **Live smoke:** `curl` `/readyz` and `/index/projects`, then `node tests/e2e/projects.mjs https://ui-production-333c.up.railway.app` (and `guests.mjs`) (allowed on this test network: it creates two test accounts; in a cloud session run `npx playwright install --with-deps chromium` first). `chat.mjs`/`governance.mjs` need the dev founders, so they only run locally. Never create accounts on the FastAPI site; the live web site may be browsed read-only to compare layouts.
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
