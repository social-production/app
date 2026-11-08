# 0086. Software projects: pull requests, merges, merge capability and the repository are proposals

Status: **accepted** (2026-11-08, cloud session; the owner steers less until parity, STATUS
2026-10-25). Plan: [`docs/plans/SOFTWARE.md`](../plans/SOFTWARE.md). Builds on
[0033](./0033-project-governance-checks.md), [0039](./0039-project-plans.md) and
[0042](./0042-project-links.md).

## Context
PARITY item 8. web's software projects (a productive project or collective service whose
production or operations plan picked the `software` subtype) show, in the Activity phase,
`ProjectSoftwareGovernancePanel` (697 lines): the repository and its mergers, "In-progress
pull requests" with three stage dots, the repository replacement history; a "Pull request"
button and a "Repository ▾" menu (`RepositoryMenu`: Replace repository, Grant or revoke merge
capability) beside Add activity; and `SoftwareGovernanceWizard` (699 lines) for submitting a
pull request, voting on it, recording a merge, and the two requests. web-backend
(`app/services/projects/software/*`, `app/routers/projects_software.py`):

- **Pull requests** (title ≤ 200, summary, PR ID ≤ 120, URL; any member files one, with no
  author vote). Stages: `approval` (members vote, 66%) → `awaiting-merge` (passed; rejected
  if it can't pass) → `confirmation` (a merge-capable member recorded a merge ID and link; the
  votes are cleared and members vote again) → `confirmed`, or back to `awaiting-merge` (merge
  cleared) if the confirmation can't pass. `replaced` when a repository replacement it
  triggered passes. The author gets `pr-approved` when the approval passes.
- **Merge capability**: the leading production/operations plan's author (`plan-creator`,
  re-synced whenever a plan takes that lead); grant/revoke requests any member files about a
  member, voted (66%); `approved-request` on a passed grant, removed on a passed revoke.
  Platform-tagged projects take the platform board instead (not parity: the board is asset
  management, owner 2026-10-31).
- **Repository**: the leading plan's `repository_url`; a replacement request (new URL, reason,
  the blocked pull request; the wizard offers pull requests in `awaiting-merge`) is voted;
  passed, it sets the leading plan's URL, marks the pull request `replaced`, removes the
  `plan-creator` and `repo-replacement` mergers and makes its author a merger
  (`repo-replacement`).
- The right rail lists pull request votes (approval and confirmation), "merge needed" actions
  for mergers, and merge capability votes; the project History tab and Past activity list
  the software decisions.

## Decision
- **One new proposal kind, `ProposalKind::Software(SoftwareRequest)`**, with
  `PullRequest { title, summary, pull_request_id, pull_request_url }`,
  `Merge { pull_request, merge_id, merge_url }` (its vote *is* the confirmation),
  `MergeCapability { agent, grant }` and `Repository { repository_url, reason,
  pull_request }`. Each is voted like any proposal (snapshot, 66%, challenges, the Full
  standing recount); web's cleared-and-restarted confirmation vote becomes a second proposal,
  since the app's votes are final (ADR 0033). No author vote, as on web.
- **Integrity** checks the fields (`sp_software::check_*`), that a merge or replacement cites
  a pull request of the same space, and that only productive projects and collective services
  take them. Who may record a merge and whether the pull request waited for one depend on
  earlier decisions, so the coordinator decides those, as the fold already does for phase
  moves and returns.
- **The coordinator replays** (`sp_software::replay`) the software proposals, their decisions
  and every time a production or operations plan took the lead (ADR 0072's lead changes, now
  with the plan's repository) in time order into web's state: stages, the recorded merge,
  mergers with web's source labels, the repository and passed replacements. A merge from a
  non-merger, or for a pull request not waiting for one, counts for nothing. `governance::
  software` is the panel's read; `propose` refuses as web-backend does ("Software governance
  requires a software project", "Only members with merge capability can record merges",
  "Pull request must be approved before merge recording", a target who isn't a member).
- **Rules in `crates/sp_software`** (not `sp_core`, which would change every DNA's hash).
- Software proposals stay off `list_proposals` (the vote dock), like plans and links: they
  have their own panel.
- **Phases** (plan): (a) records, coordinator, scenario and (b) the panel, wizard, Pull
  request button and Repository ▾ menu: *this change*, one PR; (c) the rail rows, the History tab and Past
  activity entries, and `pr-approved`.

## Consequences
- **Project integrity change: merging resets the project network** (test data only, owner
  2026-10-25). Directory and conversation are unchanged.
- An open replacement shows the repository it would replace now; web stores the one current
  when it was filed. They differ only if two replacements are open at once.
- **Seen on web, kept:** the wizard's member list for both grant and revoke is the members
  who *can't* merge (`availableMergeCapabilityCandidates`), so web can't pick a merger to
  revoke; the replacement history prints the related pull request's internal id. Owner review.
