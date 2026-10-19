# 0040. Phase returns are proposals; challenges reopen under-counted decisions

Status: accepted (2026-10-19). See docs/plans/M3-governance.md ("M3c as built"). Builds on
[0033](./0033-project-governance-checks.md) and [0039](./0039-project-plans.md); settles the
challenge window that docs/GOVERNANCE.md left open.

## Context
**Returns.** On web, members can ask to take a project back to an earlier planning phase
(`POST /projects/{slug}/revert-requests`, web-backend `projects/phases/revert.py`). The target
must be earlier than the current phase, and the Return sheet only offers web's
`revertablePhaseIds`: phases up to the second plan phase that this project shows. A reason is
required, only one return vote may be open, and opening it does not count as the proposer's Yes
(unlike advancing). When it passes, the phase changes; plans and their ratings stay.

**Challenges.** A proposal's population N is its proposer's snapshot of members (ADR 0033). A
proposer can leave members out to make N smaller and a decision easier to reach, and integrity
can't notice, because no validator can list "all members" deterministically. GOVERNANCE.md
proposed a challenge window. web has nothing like it: its server counts weekly active members
itself, so there was nothing to port.

## Decision
- **A return is `ProposalKind::Return { to, reason }`.** Integrity checks the phase id and a
  non-empty reason. Whether `to` is earlier depends on the phase reached, so the fold checks it:
  a return applies only if `to` is in `sp_governance::returns::returnable` for the phase the
  project is in when the return is decided. A return decided after another move may therefore not
  apply, and a move proposed before a return no longer applies after it (it is no longer the next
  step). The coordinator refuses returns web refuses, with web's messages, and refuses a return
  while any move or return vote is open (web's menu locks then).
- **The subtype reopens.** Returning to Proposal or the production/operations plan phase clears
  the subtype frozen by the earlier move, so the leading plan sets it again, as web's
  `sync_project_plan_leading_flags` does. Leaving that phase again cites the leader as usual.
- **A `Challenge` entry** cites a proposal, the challenger's membership and the `Members` links
  the snapshot left out. Integrity checks each link was made before the proposal, by someone the
  snapshot doesn't count, named once. Apply time adds what integrity can't see: a challenge
  counts only if it came while the proposal was open or within **48 hours of its first Outcome**
  (`sp_governance::challenge::WINDOW_MICROS`), and only if none of the members it names had left
  before the proposal (one stale name voids the whole challenge).
- **Counted challenges enlarge N.** An Outcome now cites challenges (`challenges`, serde default
  empty) and integrity recomputes its result against the snapshot plus the members they add.
  Each node takes as deciding the **earliest Outcome that cites every counted challenge**. If
  none does, the proposal is open again: its decision stops applying, members can vote, and the
  next vote (or any member's app, through `close_proposal`) writes an Outcome citing the
  challenges. The apply-time recount (ADR 0033) uses the enlarged N.
- **Decisions apply at once, as on web**, and stay challengeable for 48 hours. GOVERNANCE.md
  proposed waiting out the window before applying; that would delay every decision by two days
  compared with web. Everything a decision changes today (charter, phase, updates, plans) is
  recomputed by the fold, so a voided decision simply stops applying.

## Consequences
- Integrity changes (a new kind, entry and link type, an Outcome field): a new project DNA hash
  and a new test network (ADR 0029).
- Any member can file a challenge from the Participation tab; the app computes who is missing
  from its own view of the membership roll, so a node that hasn't synced may show nobody missing.
- Record timestamps are claimed by their authors. A member who has written nothing since could
  backdate a challenge into the window; votes already share this limit (ADR 0033). Unchanged.
- Anything irreversible that M5 adds (board receipts, purchases) must wait until a decision's
  window has closed before acting on it; `challenge_until` is in every tally view for that.
- N still counts every member, not web's weekly active members. A challenge restores members the
  proposer omitted; it can't express activity. That needs activity proofs (ADR 0033).
