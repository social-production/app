# Status: where the P2P app stands (handoff, 2026-10-19 night)

Read this first in a new chat, then `CLAUDE.md` (it has a section for cloud sessions), `README.md`
(roadmap) and `docs/CODE_RULES.md`. The workspace-level `../STATUS.md` covers web/web-backend.

## Live (test network, Railway project `social-production-p2p`)
- UI https://ui-production-333c.up.railway.app · seed node https://seed-node-production.up.railway.app (`/readyz`, `/index/projects`).
- Deployed `main`: `65b5cfd` (CI green). Pushing `main` redeploys bootstrap, seed-node and ui. CI: `gh run list --workflow ci`.
- Live features:
  - M1 identity (phrase = account, passkeys, pairing, vouches/standing, founders thebestdamn + BigBird);
  - M2 projects (create, feed/search with web's sorts, join, demand/oppose, members sheet);
  - M3a governance (propose edit/update/phase, Vote sheet, 66% of a quorum of Full members);
  - guests can browse the feed signed out;
  - usernames follow web's handle rules (no spaces).
- The M3 deploy reset the network: everyone signs in with their 24 words and re-enters a profile; founders vouch again.

## In flight: M3b plans + M4a chat, merged locally, not pushed
Neither subagent had committed; their work was committed on its branch and merged into local
`main` (`6771230`). The M3b branch is also on GitHub as `claude/m3b-plans`.
- **M4a project chat** (ADR 0038, `docs/plans/M4-talk.md`): complete. Fixed a lint error
  (`ChatPanel.svelte` dynamic delete) and rustfmt.
- **M3b plans** (ADR 0039, "M3b plans, as built" in `docs/plans/M3-governance.md`): the agent left
  no docs. The ADR and the as-built notes were written from the code; plan code references were
  renumbered from 0038 to 0039 (0038 is chat).
- Being checked now: full CI suite including all scenarios on `6771230`. Then browser tests,
  a push, and a live smoke. The deploy resets project data again (directory/people stay).
- Afterwards: `git worktree remove --force .claude/worktrees/agent-*`, delete the two
  `worktree-agent-*` branches and `origin/claude/m3b-plans`.

## Next (roadmap order)
1. Finish the push above, deploy, smoke live; update README Status.
2. M3b remainder: challenge window for under-counted populations, phase revert, links (`docs/plans/M3-governance.md`).
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
  - Run `bash scripts/e2e-local.sh tests/e2e/{accounts,projects,governance}.mjs`. Signed out, the home page is the feed; sign-up is at `/onboarding`.
- **Live smoke:** `node tests/e2e/projects.mjs https://ui-production-333c.up.railway.app` is allowed on this test network (it creates two test accounts). Never create accounts on the FastAPI site.
- **Styles:** never hand-edit `ui/src/lib/components/styles/*.css`. Add rows to `styles/PORTS` and run `python3 scripts/port-styles.py`. App-only CSS goes in `styles/additions.css` (ADR 0032).
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
