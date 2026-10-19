# Status: where the P2P app stands (handoff, 2026-10-19 evening)

Read this first in a new chat, then `CLAUDE.md`, `README.md` (roadmap) and `docs/CODE_RULES.md`.
The workspace-level `../STATUS.md` covers web/web-backend; its last section summarises this repo.

## Live (test network, Railway project `social-production-p2p`)
- UI https://ui-production-333c.up.railway.app · seed node https://seed-node-production.up.railway.app (`/readyz`, `/index/projects`).
- `main` at `4efd63d`; pushing `main` redeploys bootstrap, seed-node and ui. CI: `gh run list --workflow ci`.
- Live features:
  - M1 identity (phrase = account, passkeys, pairing, vouches/standing, founders thebestdamn + BigBird);
  - M2 projects (create, feed/search with web's sorts, join, demand/oppose, members sheet);
  - M3a governance (propose edit/update/phase, Vote sheet, 66% of a quorum of Full members);
  - guests can browse the feed signed out;
  - usernames follow web's handle rules (no spaces).
- The M3 deploy reset the network: everyone signs in with their 24 words and re-enters a profile; founders vouch again.
- `7961d27`/`4efd63d` (guest browsing, `close_proposal` rename) were waiting on CI and the redeploy when this was written. Check that CI is green and the live feed loads signed out.

## In flight: two subagent worktrees (not pushed, not merged)
Started from `f236bcc` (M3a), under `.claude/worktrees/` (git-ignored):

| Worktree branch | Work | Notes |
|---|---|---|
| `worktree-agent-a6444a03833c35003` | **M3b plans**: web's production/distribution/operations/access plans, criteria, assessments, approved-leading-plan gate for phase moves | touches `project_integrity` + governance coordinator; told to rename extern `close` → `close_proposal` like main |
| `worktree-agent-a17e01b4838e77b67` | **M4a project chat**: `discussion` zome pair in the project DNA + web's Chat tab | stays out of `project_integrity`/governance |

**To finish them:**
1. In each worktree, `git log` shows whether the agent committed.
2. Review the diff and run the checks below.
3. Merge into `main`. Conflicts are expected only in `dnas/project/dna.yaml`, `ProjectDetail.svelte` (tabs list), `docs/decisions/README.md`, `README.md` and `Cargo.toml`.
4. Rebuild (`bash scripts/build-happ.sh`), run all scenarios and the browser tests, then push.
5. Remove the worktrees: `git worktree remove --force <path>`, then `git branch -D <branch>`.

If a worktree has no commit, the agent was cut off. Read its diff and either finish it or redo it.
Both change the project DNA, so the merge deploy resets project data again (directory/people stay).

## Next (roadmap order)
1. Merge M3b + M4a (above), deploy, smoke live.
2. M3b remainder if missing: challenge window for under-counted populations, phase revert, links (`docs/plans/M3-governance.md`).
3. M4: DMs (conversation DNA), threads/posts, in-app notifications (`docs/plans/M4-talk.md` if the agent created it).
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
