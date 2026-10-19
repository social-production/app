# Status: where the P2P app stands (handoff, 2026-10-19 late night)

Read this first in a new chat, then `CLAUDE.md` (it has a **Cloud sessions** section), `README.md`
(roadmap) and `docs/CODE_RULES.md`. The workspace-level `../STATUS.md` covers web/web-backend.
**The owner is travelling on 2026-10-20/21 and will work from cloud sessions only.** Everything
below is on GitHub; nothing important lives only on the owner's PC.
**Merging:** the owner approved cloud sessions merging a PR once its CI is green (CLAUDE.md).
To start a cloud session, open claude.ai/code on `social-production/app` and say: "Read STATUS.md
and continue from the first unfinished step."

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
- Cloud work order: (1) PR #1 → merge → smoke → README/STATUS docs PR; (2) Take part layout
  (`claude/take-part-layout`); (3) `claude/m3c-reverts`: merge main, check it invented nothing
  (its app-only ChallengeList must be flagged to the owner, since web-backend has no challenge
  window), open a PR.

## In flight (cloud session 2026-10-19)
- PR #1's first CI run was cancelled at its 60-minute job timeout while the scenarios were
  still passing (governance ~21 min, health ~10). The timeout is now 150 min (`ci.yml`); waiting
  for green to merge.
- `claude/take-part-layout` (Step 2, pushed, no PR until PR #1 merges): ParticipationPanel now
  follows web's ProjectLifecyclePanel: stepper → `DetailPhaseContext` (kicker, title, summary from
  web-backend's `lifecycle_copy.py`, in `domain/phaseCopy.ts`) → `.phase-panel.overview-phase-work`
  with the plans and `PhaseChangeSection` (web's gate note + `PhaseMenu`). Styles ported via PORTS.
  Then web's own Take part dock (`ParticipationDock`: Join for non-members, Vote for phase
  proposals; the Details dock now only votes on edits/updates).
  Checked: `npm run check/lint -w ui`, boundaries, sizes. **Not yet done:** a 390px screenshot
  against web (needs the local seed node, i.e. Nix; this cloud environment's network policy
  blocks holochain-ci.cachix.org, GitHub tarballs and the live web site, so it couldn't run here); **Gap vs web:** web's Advance opens a sheet asking for a reason; our phase proposals have
  no reason field (integrity change), so the menu item files at once. M3c's Return already carries
  a reason: when M3c merges, add `reason` to `Phase` in the same integrity bump and port the
  sheet. M3c's "Return to an earlier phase" button should become a PhaseMenu item (web does).
- `claude/m3c-reverts` (Step 3, reviewed, not merged/PR'd yet; waits for PR #1 and Step 2):
  `ReturnSheet` is a faithful port of web's revert sheet (target select, reason, web's copy).
  Two things to fix when merging main into it: its standalone "Return to an earlier phase"
  button must become the `return` item of `PhaseMenu` (web's `phaseMenuItems`), and
  `ChallengeList` ("Counts you can challenge") is **app-only** (web-backend has no challenge
  window). **Choice made (owner to review):** following "port only from web", the M3c PR drops
  the ChallengeList UI and keeps the challenge rules (zome, `sp_governance`, scenarios, ADR 0040),
  so the mechanism is ready when a challenge screen is designed or found in web. Undo by
  restoring `ChallengeList.svelte` from `a0ce970`.

## Cloud environment notes
- The proxy blocks GitHub tarball downloads for repos outside the session, which `nix develop`
  needs for flake inputs. Workaround: fetch them over git with `--override-input <path>
  'git+https://github.com/<owner>/<repo>?rev=<locked rev>&shallow=1'` for every GitHub node in
  `flake.lock` (plus `--no-write-lock-file`); the Holochain cachix cache is reachable once
  holochain-ci.cachix.org is in the environment's allowed domains.

## Live (test network, Railway project `social-production-p2p`)
- UI https://ui-production-333c.up.railway.app · seed node https://seed-node-production.up.railway.app (`/readyz`, `/index/projects`).
- Deployed `main`: `65b5cfd`. Pushing/merging to `main` redeploys bootstrap, seed-node and ui.
- Live: M1 identity, M2 projects, M3a governance (edit/update/phase proposals, Vote sheet), guests
  browse the feed signed out, usernames without spaces. Founders thebestdamn + BigBird.

## Step 1: PR #1 (`claude/m3b-m4a`), M3b plans + M4a project chat
https://github.com/social-production/app/pull/1. This is local `main` (`2a2b749`), not yet on
`origin/main`.
- Contents: plans (ADR 0039, "M3b plans, as built" in `docs/plans/M3-governance.md`); project
  chat (ADR 0038, `docs/plans/M4-talk.md`); browser tests `tests/e2e/chat.mjs` and the plan flow in
  `tests/e2e/governance.mjs`; fixes (dock over the chat on phones, chat Delete crash,
  `javascript:` plan links, lost quick ratings/votes).
- Checked locally: everything except the scenarios, which were stopped for CPU. PR CI runs them
  (`gh pr checks 1`; ~60 min).
- **To finish:** CI green → `gh pr merge 1 --merge` (the 2026-10-19 local session may have armed a watcher that merges it automatically when CI passes; check `gh pr view 1`) (this deploys; project data resets again,
  people stay) → wait for `/readyz` 200 and the UI → smoke (see Live smoke below) → update
  README Status + roadmap (M3: plans live; M4: chat live) and this file. If CI fails, fix on the
  branch, push, and wait again.

## Step 2: fix the Take part tab layout (owner spotted it; not done)
At 390px our Participation ("Take part") tab shows the phase title, plans, "Add plan" and the
"Propose moving" button **above** the phase stepper. web's (live:
https://web-production-885ec.up.railway.app/projects/brunswick-tool-library-pilot-a47976e5?tab=participation)
shows, top to bottom: stepper → "CURRENT PHASE" kicker + phase title + one-line phase summary →
plan cards.
- Cause: web's `.participation-tab` is a flex column whose children are placed by `order`
  (`styles/detail.css` ~l.790: `.overview-phase-tabs` 1, `.phase-title` 2,
  `.overview-phase-context` 3, `.pending-votes-panel` 4, `.overview-phase-work` 5;
  `.overview-governance` is `display: contents`). Our `ParticipationPanel.svelte` wraps the work
  in a hand-written `<section class="phase-panel">` with no order class, so it gets order 0 and
  lands above the stepper (order 1).
- Fix: port web's structure from `web/src/lib/features/projects/detail/ProjectLifecyclePanel.svelte`
  (~l.1588–1745): `ProjectLifecyclePhaseTabs`, then `DetailPhaseContext`
  (`web/src/lib/features/detail/DetailPhaseContext.svelte`: kicker "Current phase"/"Complete"/
  "Upcoming", title, summary), then `<section class="phase-panel overview-phase-work">` with the
  plans, then web's `ProjectPhaseChangeSection`
  (`web/src/lib/features/projects/detail/components/ProjectPhaseChangeSection.svelte`) for the
  phase-move button instead of our hand-made one. Phase summaries come from web's lifecycle data.
- Also: web gives the Take part tab its **own** `DetailActionDock` (join split button + a
  VoteDockControl for participation votes, no Edit, no signals). Ours shows the Details dock
  (with Edit) on every tab except Chat.
- Screenshot both at 390px before and after (Playwright, `device({ viewport: { width: 390, height: 844 } })`).

## Step 3: agent branches (WIP, pushed, not reviewed)
Both started from `67b54f0` and were told not to run scenarios. They may have pushed more commits
after this was written; `git log origin/main..origin/<branch>`.
- `claude/m3c-reverts` (`a0ce970`, **finished** per its agent; scenarios not run): phase
  returns ported from web-backend `projects/phases/revert.py` (`ProposalKind::Return { to, reason }`,
  `sp_governance::returns`, web's ReturnSheet in the Participation tab); the challenge window
  (ADR 0040: `Challenge` entry, `Outcome.challenges`, 48 h after the first Outcome, reopens a
  decision whose count left a member out; web-backend has no challenge window, so it follows
  GOVERNANCE.md). Decisions still apply at once. Left: plan-card challenge button, browser tests
  for return/challenge, weekly-active counting. Note: `scripts/port-styles.py` finds `web/`
  relative to the checkout, so it doesn't work from a worktree. Before merging, run scenarios
  `reverts.rs`, `challenges.rs`, `governance.rs`, `plans.rs` (outcome/vote/close changed).
- `claude/m4b-threads` (**parked, do not merge**, see Owner decisions): discussion threads/posts/comments/votes in the `discussion` zome + web's
  screens (ADR 0041, "M4b as built" in `docs/plans/M4-talk.md`).
- To finish each, after PR #1 is merged: rebase or merge `main` into it, read the diff (look out
  for hand-written layout where web has a component, as in Step 2), run the checks, add a browser
  test, open a PR, and merge when CI is green. Check whether the as-built section lists what is left.

## Known gaps
- Plan rating/vote errors go to `detail.error`, which is only shown when the project fails to
  load, so a refused write is invisible. The optimistic plan sheet doesn't roll back, and leaving
  the page before queued writes finish cancels them.
- Guests can't open project pages yet (only the feed). No @mentions, photos or unread markers in chat.

## Next (roadmap order)
1. Steps 1–3 above.
2. M3 remainder after M3c: links between items (`docs/plans/M3-governance.md`).
3. M4: DMs (conversation DNA), threads/posts, in-app notifications (`docs/plans/M4-talk.md`: M4b threads, M4c DMs, M4d notifications).
4. Resilience (ADR 0036/0037): move archivist + index out of the gateway into a crate any full node runs; desktop Storage setting (light / slice / archive + GB limit); sharding switch at ~30 always-on full nodes, 20+ copies; phones light; Android-only "help while charging" later.
5. A public read-only project page for guests (the node would serve it; today project pages need sign-in).

## How to work (things learned the hard way)
- **Commands:** run them inside nix, `nix develop --accept-flake-config --command bash -c '…'` (PATH may need `/nix/var/nix/profiles/default/bin`). The path has a space; quote it.
- **Fast checks:** `bash scripts/check-boundaries.sh`, `bash scripts/check-sizes.sh`, `npm run check -w ui`, `npm run lint -w ui`, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --exclude scenarios`. Look for `error: test failed`, not just `FAILED`.
- **Scenarios:** `bash scripts/build-happ.sh`, then `cargo test -p scenarios -- --test-threads=1` (slow: ~10–15 min per file).
  - Settle each DNA's cells separately with `await_consistency`.
  - Never depend on two integrity crates in one test binary; mirror types with serde instead.
- **Browser tests:**
  - Start the local seed node: `RP_ORIGIN=http://localhost:4174 bash scripts/dev-seed.sh`. It builds a dev hApp whose founders are `tests/e2e/fixtures/dev-founders.json`, so governance can be tested.
  - Wait for "zomes compiled" in its log.
  - Run `bash scripts/e2e-local.sh tests/e2e/{accounts,projects,chat,governance}.mjs`. Signed out, the home page is the feed; sign-up is at `/onboarding`.
- **Live smoke:** `curl` `/readyz` and `/index/projects`, then `node tests/e2e/projects.mjs https://ui-production-333c.up.railway.app` (allowed on this test network: it creates two test accounts; in a cloud session run `npx playwright install --with-deps chromium` first). `chat.mjs`/`governance.mjs` need the dev founders, so they only run locally. Never create accounts on the FastAPI site; the live web site may be browsed read-only to compare layouts.
- **Styles:** never hand-edit `ui/src/lib/components/styles/*.css`. Add rows to `styles/PORTS` and run `python3 scripts/port-styles.py`. App-only CSS goes in `styles/additions.css` (ADR 0032).
- **One write at a time per person:** two zome calls that write to the same source chain at once are refused (the chain head moved). Optimistic UI that fires several writes (ratings, votes) must queue them, as `DetailState.inTurn` does.
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
