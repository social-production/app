# Status: where the P2P app stands (updated 2026-10-21 by a cloud session; see the newest "Done")

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

## Done 2026-10-21: Messages unread badge, PR #19 merged (~07:27 UTC / 6:27 pm Melbourne), not yet on the live UI (see Live)
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
   The members count in a project chat's header is text (web opens a members panel).
7. **Group "Delete" leaves the group** (ADR 0045), with the "left the group" line and a
   handover; web's Delete drops the membership silently and can leave a group without its
   manager. Removed people can still read what the chat gossips (they know its seed).

8. **Notifications leave out "Plan became leading"** (ADR 0046): web sends it only to people
   who turn it on in settings, and the app has no notification settings yet.

## Live (test network, Railway project `social-production-p2p`)
- UI https://ui-production-333c.up.railway.app · seed node https://seed-node-production.up.railway.app (`/readyz`, `/index/projects`).
- Deployed `main`: `dbb1621` (PR #19 merged 2026-10-21 ~07:27 UTC, 6:27 pm Melbourne). Merging to `main` should redeploy bootstrap, seed-node and ui. **Check that the UI actually changed before smoking:** on 2026-10-21 PR #19 (`dbb1621`, merged 07:27 UTC, `main` CI green 08:24 UTC) had still not reached the live UI at 09:14 UTC (its `index.html` was last modified 05:46 UTC; the new code's `messageCount` prop was absent from every chunk). Railway shows no `ui` deployment for that merge (its last one is PR #17's), so the merge never triggered one. Cloud sessions can't see Railway yet: the owner will add a Railway **project token** as `RAILWAY_TOKEN` in the cloud environment (the API host `backboard.railway.com` is already reachable); Railway's Redeploy rebuilds that deployment's own commit (PR #17's), so it can't help; use "Deploy latest commit" on `ui`, or a fresh merge to `main`. Note that a merge landing while `main`'s CI runs cancels that run (concurrency).
- Live: M1 identity, M2 projects, M3a governance (edit/update/phase proposals, Vote sheet), M3b
  plans, M3c phase returns (+ challenge rules), M3d project values, M3e links between projects (web's Links tab), M4a project chat, M4c Messages (direct and group chats, Public tab, group options, chat menu, edits and pins), M4d notifications (phase votes, phase changes, chat replies; bell count), web's Take part tab, guests browse the feed signed out, usernames without spaces. Founders
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
- **Seed-node memory (2026-10-21):** the owner saw ~3.2 GB in Railway, against a usual ~0.8 GB. It was healthy then (`/readyz` 0.3 s). Unconfirmed guess: every live test account and its DM/group chat cells stay hosted and loaded (each chat is a clone cell per member). Check it with Railway metrics and logs once `RAILWAY_TOKEN` is set. Cut live test runs that create accounts until then.
- Plan rating/vote errors go to `detail.error`, which is only shown when the project fails to
  load, so a refused write is invisible. The optimistic plan sheet doesn't roll back, and leaving
  the page before queued writes finish cancels them.
- Guests can't open project pages yet (only the feed). No @mentions, photos or unread markers in chat.
- Challenges have no screen (see "For the owner to review").

## Next (roadmap order)
Port each screen from web (Owner decisions); compare at 390px with live web first.
1. M4, from web: @mentions in chat (+ `mention` notifications), scroll
   to the linked chat message. Still from web later: attachments, calls, "seen" receipts, the project members panel.
   Project-chat read marks and list settings move into private entries with the next
   project-DNA integrity change. Threads stay parked (Owner decisions).
2. Resilience (ADR 0036/0037): move archivist + index out of the gateway into a crate any full node runs; desktop Storage setting (light / slice / archive + GB limit); sharding switch at ~30 always-on full nodes, 20+ copies; phones light; Android-only "help while charging" later.
3. A public read-only project page for guests (the node would serve it; today project pages need sign-in).
4. CI: scenarios run in 4 shards (~55 min); add shards if one nears 120 min.

## How to work (things learned the hard way)
- **Commands:** run them inside nix, `nix develop --accept-flake-config --command bash -c '…'` (PATH may need `/nix/var/nix/profiles/default/bin`). The path has a space; quote it.
- **Fast checks:** `bash scripts/check-boundaries.sh`, `bash scripts/check-sizes.sh`, `npm run check -w ui`, `npm run lint -w ui`, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --exclude scenarios`. Look for `error: test failed`, not just `FAILED`.
- **Scenarios:** `bash scripts/build-happ.sh`, then `bash scripts/check-scenarios.sh` (slow: ~10–15 min per file; `cargo test -p scenarios --test <file> -- --test-threads=1` for one). In a cloud session use `bash scripts/cloud-scenarios.sh`: without its no-IPv6 shim every conductor fails with "Address family not supported by protocol".
  - **Disk:** a debug build of the whole workspace is ~24 GB and fills a cloud session's disk. Build with `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0` (the cloud scripts do), and never mix the stock `cargo` with the Nix one in one `target/` (two full builds); `rm -rf target/debug` if `df` shows the disk filling.
  - Settle each DNA's cells separately with `await_consistency`.
  - Never depend on two integrity crates in one test binary; mirror types with serde instead.
  - **End every test with an awaited shutdown:** `common::shut_down(&mut conductors).await`, or `net.shut_down().await` / `pair.shut_down().await`. sweettest's `Drop` only spawns the shutdown on the test's runtime, so the last test's conductors could still be tearing down when the process exited. In CI the binary then crashed after every test had passed: SIGSEGV after `plans` (PR #13), and `malloc_consolidate(): unaligned fastbin chunk` after `groups` (PR #17).
- **Browser tests:**
  - Start the local seed node: `RP_ORIGIN=http://localhost:4174 bash scripts/dev-seed.sh`. It builds a dev hApp whose founders are `tests/e2e/fixtures/dev-founders.json`, so governance can be tested.
  - Wait for "zomes compiled" in its log.
  - Run `bash scripts/e2e-local.sh tests/e2e/{accounts,projects,chat,governance}.mjs`. Signed out, the home page is the feed; sign-up is at `/onboarding`.
- **Live smoke:** `curl` `/readyz` and `/index/projects`, then `node tests/e2e/projects.mjs https://ui-production-333c.up.railway.app` (allowed on this test network: it creates two test accounts; in a cloud session run `npx playwright install --with-deps chromium` first). `chat.mjs`/`governance.mjs` need the dev founders, so they only run locally. Never create accounts on the FastAPI site; the live web site may be browsed read-only to compare layouts.
- **Styles:** never hand-edit `ui/src/lib/components/styles/*.css`. Add rows to `styles/PORTS` and run `python3 scripts/port-styles.py`. App-only CSS goes in `styles/additions.css` (ADR 0032).
- **One write at a time per person:** two zome calls that write to the same source chain at once are refused (the chain head moved). Optimistic UI that fires several writes (ratings, votes) must queue them, as `DetailState.inTurn` does.
- **A write isn't in your own network reads at once:** after a vote, `tally.my_vote` can still be
  empty for a while; UI flows keep what they just wrote (as `LinksState.cast` does).
- **Debugging the UI:** errors caught into `detail.error` are not shown once the project has loaded; log them temporarily, or find them with a Playwright script that listens to `console`. When stopping a preview server, kill it by port (`ss -ltnp | grep :4174`). `pkill -f "<its command>"` also matches your own shell and kills it.
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
