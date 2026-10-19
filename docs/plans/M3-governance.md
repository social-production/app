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
