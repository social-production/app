# 0087. Software votes outside the panel: the rail, Take part's vote sheet, History and `pr-approved`

Status: **accepted** (2026-11-09, cloud session; the owner steers less until parity, STATUS
2026-10-25). Plan: [`docs/plans/SOFTWARE.md`](../plans/SOFTWARE.md) phase (c). Builds on
[0086](./0086-software-projects.md), [0046](./0046-notifications-derived-on-device.md) and the rail's port
(STATUS 2026-10-27).

## Context
ADR 0086 shipped web's software panel. web also shows a software project's decisions
elsewhere: the right rail (web-backend `bootstrap/activity_rail.py`: "Vote · pull request",
"Action · merge needed" with Record merge, "Vote · merge capability", "Vote · repository";
`activity_rail_history.py`: the votes you cast), Take part's pending votes
(`pushSoftwareGovernanceActions`: merge capability and repository votes are cast there; the
panel has no button for them), the History tab and the Activity phase's Past activity
(`build_software_history_entries`: "Pull request approval", "Merge confirmation", "Merge
capability change", "Repository replacement"), and `pr-approved` to a pull request's author.
Until this change the app had **no way to vote on a merge capability or repository request**.

## Decision
- **One read.** Everything comes from `governance::software` (the panel's read). The
  coordinator adds only `approved: { at, by }` to each pull request (when its own vote passed
  and whose Outcome closed it), for `pr-approved`. Coordinator only: no network reset.
- **A `Software` proposal kind in the UI** (`domain/governance.ts`: `{ label, title,
  description, reason }`, web's generic `VoteDecisionBody` shape) wraps a merge capability or
  repository request (or a pull request's current vote, in History), so the existing vote
  sheet, VoteDock and History cards show and cast it (`domain/softwareVotes.ts`). Votes go
  through `cast_vote` like any other; the zome's list stays without them.
- **The page reads software once.** `DetailState.software` (a `SoftwareState`) loads with the
  page for software projects, so the panel, Take part's vote sheet (merge capability and
  repository votes after the phase votes), History and Past activity share it. Its load closes
  settled software votes, as the page does for proposals (ADR 0033).
- **Links.** `?open=vote&voteKind=pull_request&assess=1` opens the pull request's vote wizard,
  `voteKind=pull_request_merge` its Record merge (rail and `pr-approved`), as web's
  `softwareWizardRequest`; without an action the card scrolls into view. Merge capability and
  repository links open Take part's vote sheet on that request.
- **History and Past activity** share web's `DecisionHistoryCard`, now its own component
  (`components/governance/DecisionHistoryCard.svelte`, with its sheet); Past activity
  interleaves software decisions with ended activities, newest first, as web's unified history.
- **The rail** reads `software` for software projects it already reads, adds web's four rows
  (three of each kind, newest first) and its History casts (`domain/softwareRail.ts`), and now
  keeps web-backend's cap of **eight vote rows in all, newest first** (it had no cap).
- **Rail sections open once, when their first cards arrive** (`RailSection`, web's
  `railOpenInitialized`), not only when they have cards at mount: the rail now reads more per
  project and its cards can land after it opens (web's arrive with the page's bootstrap).
- **`pr-approved`** (`domain/softwareNotifications.ts`, category Votes needed) is derived on
  the device like every card (ADR 0046): to the author, at the approval's close.

## Consequences
- The rail reads `software` for each software project on every reload; a founder in many
  test projects waits ~10 s for it locally. If it gets slow live, read it only for projects in
  their Activity phase.
- A pull request's History row and rail History cast show its *current* vote (the merge
  confirmation once one is recorded), as the panel does; web keeps one row per pull request too.
- **Seen on web, changed:** web's rail History leaves a confirmed merge "Still open" (its
  `_vote_outcome` doesn't list `confirmed`); the app says "Passed". Owner review.
- Take part's sheet lists merge capability and repository votes; web also lists pull request
  rows there with an Assess button, which the app leaves to the panel's own Assess (the
  sheet casts votes, it doesn't open wizards). Owner review.
