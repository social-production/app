# M3 — Governance (plan, 2026-10-19)

Decisions inside a project become records any node can check: **Proposal → Vote → Outcome**
(ADR 0009). Decided autonomously while the owner was away; the reasoning is in ADR 0033.

## M3a — proposals, votes, outcomes (this batch)
- **project DNA, new `governance` zome** (integrity + coordinator):
  - `Proposal { kind, members }`. The kinds: `Edit { title?, description?, location? }` (only what
    changes, as web stores it), `Update { body }`, `Phase { to }`.
    `members` is the snapshot of `Members` link actions the population N is counted from.
  - `Vote { proposal, membership, choice: Yes | No | Neutral }`. One vote per agent per proposal,
    final once cast. The voter cites their own `Members` link.
  - `Outcome { proposal, votes, passed }`. Any member can write it. Validators re-fetch every cited
    vote, drop duplicates and neutrals, and recompute `sp_governance::tally` against
    `required_votes(N)` at 66%. A false claim is rejected.
  - The proposer's own Yes is cast with the proposal (web's proposer vote). When that alone is
    enough (small N), the coordinator writes the Outcome at once, so it "applies straight away".
- **Apply rules (read time):** current charter = creation charter + passed `Edit`s in order;
  phase = passed `Phase` outcomes; updates = passed `Update`s. Before applying, each node checks
  through its directory cell that every counted voter held Full standing before the proposal
  (ADR 0033).
- **Phases** are ported from web-backend `projects/phases/labels.py` into `sp_governance::phases`
  (per mode: Proposal → Production/Operations Plan → Distribution/Access Plan → Activity → Closed).
  Leaving Proposal needs 66% demand among signals (web's signal gate, checked at apply time).
  Later phases need an approved leading plan, which arrives with M3b.
- The creator's direct charter edit (M2) ends: charter updates are rejected by integrity, and edits
  go through proposals. The feed index reads title and summary from the project's current
  charter, not from the listing.
- **UI from web:** VoteDockControl (pending votes in the action dock), ProposeEditSheet in proposal
  mode, AddUpdateSheet, the phase stepper, and the Participation tab.
- **Tests:** sp_governance units (phases, apply order); scenarios (proposal → votes → outcome,
  forged outcome rejected, double vote, non-member vote rejected, edit applied, phase gate);
  browser test (propose an edit, the second member votes, it applies).

## M3b — plans and challenges (next)
Production/distribution/operations/access plans with criteria and assessments (web's
`projects_plans.py`, `plan_criteria.py`); the challenge window for under-counted populations
(GOVERNANCE.md); revert of phase changes; links between items.

## M3a as built (2026-10-19)
- Proposals, votes and outcomes live in `project_integrity` (one zome, so governance can check
  `Members` links); the API is the new `governance` coordinator, which calls the `project` zome for
  membership, signals and the base charter, and the directory cell for standing.
- Phases come from `sp_governance::phases` (ported from web-backend) and are returned with the
  project view, so the UI never re-implements them.
- UI from web: VoteDockControl (Vote button + sheet), VoteDecisionBody, VoteChoiceBar,
  ProposeEditSheet (proposal wording), AddUpdateSheet, ProjectUpdatesSection, DetailPhaseStepper,
  and the Participation tab.
- Differences from web: votes are final (no clearing, ADR 0033); personal-service projects take
  no proposals yet (their direct updates come with M5's service work).
- Tests: `cargo test -p sp_governance` (phases, verdicts); `tests/scenarios/tests/governance.rs`
  (edit passes and applies, one vote per agent, rejected and New-voter proposals change nothing,
  phase gate and step-by-step moves); `tests/e2e/governance.mjs` with the dev founders that
  `scripts/dev-seed.sh` builds into a local hApp.
- Also in this batch: usernames follow web's handle rules (`sp_core::handle`), the lair message
  limit (ADR 0034), phrase restore fallback (ADR 0035), full arc on desktop nodes (ADR 0036), and
  ported styles regenerated from `styles/PORTS`.

## M3b plans, as built (2026-10-19)
- `project_integrity`: `ProposalKind::Plan(Plan)` (web's wizard fields: kind, title, description,
  demand note, subtype, repository, location, recipients, 1–30 stages), `ProposalKind::Phase`
  gains `plan: Option<ActionHash>`, a `Rating` entry and the `ProposalToRatings` link (ADR 0039).
  Field limits in `sp_core::plan`.
- `sp_governance::plans` (ported from web-backend `projects_plans.py`, `plan_criteria.py`,
  `phases/gates.py`): plan kinds per mode and phase, the rubric, the required plan kind for leaving
  a phase, and `pick_leader` with exact fractions. Tests in `plans_tests.rs`.
- Governance coordinator: `propose` takes plans (only in the matching phase and mode) and fills in
  the leading plan on phase moves; `rate`; `list_plans` returns the plan board (plans with their
  rubric averages, your own ratings, the leader per kind and the current gate). The fold in
  `apply.rs` applies a move only with an approved cited plan of the right kind, and takes its
  subtype.
- UI from web: ProjectLifecyclePlanPhaseContent, PhaseEmptyState, CollapsiblePlanCard,
  PlanCriterionRatingRow, PlanWizardShell, PlanCreationWizard (with the direct-use and software
  licence notices), in `components/plans/` and `features/projects/PlanPhase.svelte`, shown in the
  Participation tab for the selected phase. Styles: `styles/plans.css`, `styles/wizard.css`.
- Tests: `tests/scenarios/tests/plans.rs` (an approved plan lets the project leave its plan phase;
  ratings pick the leader and its subtype shapes the phases); `tests/e2e/governance.mjs` files a
  production plan in the wizard, both founders rate and approve it, and the project moves on.
- The plan sheet updates at once and queues its writes (`DetailState.inTurn`): concurrent writes
  to one source chain are refused by the conductor.
- Not yet: the challenge window for under-counted populations, phase reverts, links between
  projects, value criteria (with project values), showing refused rating/vote writes in the sheet.

## M3c as built (2026-10-19)
Phase returns from web, and the challenge window for under-counted populations (ADR 0040).
- **Rules** (`crates/sp_governance`): `returns` (web's `revertablePhaseIds`: earlier phases up to
  the second plan phase that the project shows; returning to Proposal or the first plan phase
  reopens the subtype) and `challenge` (the 48-hour window after a proposal's first Outcome, the
  members challenges add, and which Outcome decides a challenged proposal). Unit tests in both.
- **`project_integrity`**: `ProposalKind::Phase` gains web's required `reason` (web's advance and
  close sheets ask "why"; the vote sheet shows it), `ProposalKind::Return { to, reason }`, a `Challenge` entry
  (proposal, challenger's membership, omitted `Members` links) with the `ProposalToChallenges`
  link, and `Outcome.challenges`: integrity counts N as the snapshot plus the members the cited
  challenges add.
- **Governance coordinator**: `propose` takes returns (web's checks and messages, no proposer
  Yes, one phase vote at a time); `challenge` cites everyone the count leaves out. New modules:
  `roll` (the membership roll with leave times), `count` (population, deciding Outcome, who is
  missing), `decide` (the recount, moved out of `apply`), `moves` (move and return checks). The
  fold in `apply.rs` applies a return only if it still goes back from the phase reached. Tally
  views carry `added`, `missing`, `challenge_until` and `can_challenge`; the project view carries
  `returns`.
- **UI**: web's return composer (`ReturnSheet`, styles ported from
  `ProjectPhaseChangeSection.svelte` into `governance.css` as `.return-sheet`), opened from the
  "Return to an earlier phase" item of web's Phase ▾ menu (`PhaseChangeSection`), with web's lock
  reason while a phase vote is open. The vote sheet shows a return's "Why" and notes a challenged
  or incomplete count.
- **Differences from web**:
  - web has no challenges at all. They exist here because no server counts the population.
    There is no screen for them yet: an app-only `ChallengeList` was built, then dropped to keep
    to "port only from web" (owner to decide; restore it from `a0ce970`). Challenges are reachable
    through the zome (`challenge`) and the scenarios.
  - Decisions apply at once and stay challengeable for 48 hours, rather than waiting out the
    window as GOVERNANCE.md first proposed.
- **Tests**:
  - `cargo test -p sp_governance` (returns, challenge window).
  - `tests/scenarios/tests/reverts.rs`: the return is offered and refused as on web, comes
    without the proposer's Yes, one at a time, applies, and the project moves on again.
  - `tests/scenarios/tests/challenges.rs`: a member who was offline when a rename was proposed
    challenges it. It stops applying, reopens with N = 3, and applies again after their vote.
  - Not run in this batch: the coordinating session runs them after `scripts/build-happ.sh`.
    No browser test for the return sheet yet.
- **Left**:
  - activity-based N (web's weekly active members) needs activity proofs;
  - plan cards don't offer the challenge button yet (the zome takes challenges on plans, and
    `list_plans` returns the same fields);
  - web's `revertHistory` (stored, never shown on web) and personal services' direct
    "Return to active" (M5);
  - irreversible M5 actions must wait for `challenge_until`;
  - browser tests for the return sheet and the challenge list;
  - links between projects and value criteria (from M3b's list).

## M3d values, as built (2026-10-19)
web's Proposal-phase values (ADR 0041).
- **Rules** (`sp_governance::values`): importance 1–10, averages in hundredths, prominence
  (average ≥ 5), value criterion ids (`value:<hash>`) and web's question, `label_key` for
  duplicates. Unit tests in the module.
- **`project_integrity`**: `Value { label, membership }` (from the "values" anchor, `Values`
  link) and `Importance { value, membership, importance }` (`ValueToImportance` link). A plan
  `Rating` may name any of the project's values as its criterion.
- **Governance coordinator** (`values.rs`): `add_value`, `rate_value` (Proposal phase only,
  web's messages; a label already listed is refused) and `list_values` (latest vote per member,
  counted from members with Full standing when they voted). `list_plans` adds the prominent
  values to each plan's criteria (`CriterionView.kind`: `Rubric` | `Value`); `rate` accepts
  them; a plan's score averages every rated criterion (web's `plan_average_rating`).
- **UI from web**: ProductiveLifecyclePhaseOne (`features/projects/ValuePhase.svelte`, shown for
  the Proposal phase of productive and collective-service projects), ProjectValueCard
  (`components/values/ValueCard.svelte`), DiscreteScale, AddValueSheet (with a warning line for
  refusals, an app addition), and web's "Shared value" label on value criteria in the plan sheet.
  Styles: `styles/values.css` (PORTS).
- **Tests**: `cargo test -p sp_governance values`; `tests/scenarios/tests/values.rs`
  (duplicates and blanks refused, latest vote counts, a New member's vote doesn't, Proposal
  phase only, a prominent value becomes a plan criterion and is rated, a non-value criterion is
  refused).
- **Left**: personal services' Proposal content (availability, requests) is M5; links between
  items.
