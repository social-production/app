# Software projects (PARITY item 8, ADR 0086)

Port web's software governance: `ProjectSoftwareGovernancePanel.svelte`,
`SoftwareGovernanceWizard.svelte`, `RepositoryMenu.svelte` (web
`src/lib/features/projects/detail/components/`), shown in a software project's Activity phase
(`ProductiveLifecyclePhaseFive`, `ProductiveLifecyclePhaseSix`, `CollectiveServicePhaseFive`),
over web-backend's `app/services/projects/software/*`. Not parity: platform-tagged projects'
board mergers (asset management, owner 2026-10-31).

## Phase (a): records and coordinator (project reset)
- [x] `crates/sp_software`: field checks, `Stage` with web's labels, `replay` (stages, merges,
  mergers with web's source labels, the repository, passed replacements) with unit tests.
- [x] Integrity: `ProposalKind::Software(SoftwareRequest)` (`software.rs`): fields, a merge or
  replacement cites a pull request of the same space, productive/collective-service only.
- [x] Governance coordinator: `software.rs` (ledger: software proposals with their decisions
  and the production/operations lead changes, replayed; `propose`'s refusals),
  `software_view.rs` (`software` extern: web's `ProjectSoftwareGovernanceDataOut` in the
  app's shape); no author vote; off `list_proposals`.
- [x] Scenario `tests/scenarios/tests/software.rs`.

## Phase (b): the panel (UI only; shipped with (a) in one PR)
- [x] `services/software.ts`, `domain/software.ts`; `features/projects/software/` (state, panel,
  wizard + `softwareWizard.ts`, `PullRequestReview`, `RepositoryMenu`, `SoftwareDock`).
- [x] Port the panel (repository card, mergers, in-progress pull requests with stage dots and
  chevrons, replacement history), the wizard (five modes in `PlanWizardShell`'s port) and
  `RepositoryMenu` (on the app's `PhaseMenu` port), with the "Pull request" round-plus button
  beside the calendar's Add activity (web's `dock-extra` slot), only for software projects
  (the charter's subtype, which the leading plan sets); web's placeholder ("Pull request tools
  appear here once a leading software plan is approved…") while the panel hasn't loaded.
- [x] Styles via PORTS (`python3 scripts/port-styles.py`), never by hand.
- [x] Browser test `tests/e2e/software.mjs`: plan → PR → vote → record merge → confirm.

## Phase (c): elsewhere on web (coordinator + UI, ADR 0087)
- [x] Right rail: "Vote · pull request" (approval and confirmation, Assess opens the vote
  wizard), "Action · merge needed" with Record merge, "Vote · merge capability", "Vote ·
  repository" (web's `activity_rail.py`, `RightRailPanel`), and their History casts
  (`domain/softwareRail.ts`); the rail now keeps web's eight vote rows, newest first.
- [x] Take part's vote sheet casts merge capability and repository votes (web's
  `pushSoftwareGovernanceActions`; the app had no way to vote on them before).
- [x] History tab and Past activity: web's `project-pull-request-approval`,
  `-confirmation`, `project-merge-capability-change`, `project-repository-replacement` cards
  (`build_software_history_entries`, `domain/softwareVotes.ts`), on the extracted
  `DecisionHistoryCard`; Past activity interleaves them with ended activities.
- [x] `pr-approved` notification to the pull request's author (Votes needed), opening Record
  merge (`?open=vote&voteKind=pull_request_merge`); the coordinator's pull request view adds
  `approved: { at, by }`.
- [x] Links: `voteKind=pull_request&assess=1` → vote wizard, `pull_request_merge` → Record
  merge (`software/softwareLink.ts`).
- [x] Browser test `software.mjs` extended (rail rows, Assess from the rail, `pr-approved`,
  a merge capability vote from the rail, History and Past activity); scenario `software`
  checks `approved`.
