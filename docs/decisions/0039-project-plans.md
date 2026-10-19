# 0039. Plans are proposals; ratings pick the leader; phase moves cite it

Status: accepted (2026-10-19). See docs/plans/M3-governance.md. Builds on
[0033](./0033-project-governance-checks.md).

## Context
On web, a project leaves its planning phases only with an approved leading plan
(`_ensure_project_phase_plan_gate`). Members submit production, distribution, operations
("organisation") or access plans in the matching phase, vote them through, and rate them on a
rubric (`plan_criteria.py`). The leader of each kind is the plan with the best approval ratio,
then the best average rating; a tie on both means no leader. A production or operations plan
also chooses the project's subtype, which changes the phases that follow (software skips
distribution). Every node has to reach the same verdict without a server.

## Decision
- **A plan is a `ProposalKind::Plan`.** It is approved by the usual Vote/Outcome maths (66% of a
  quorum of Full members, ADR 0033), so there is no second voting system. The author's Yes is not
  added automatically for plans: plans are assessed first, then voted on, as on web.
- **Ratings are `Rating` entries** (proposal, membership, criterion, 1–5 or `None` to clear),
  linked from the plan. The latest rating per member per criterion counts, and only ratings from
  members who had Full standing before the plan was filed. Ratings only rank plans that are
  already approved; they never decide approval, so they stay out of Outcome validation and may
  change.
- **Leader maths is integer-only** (`sp_governance::plans`): approval ratios and averages are
  compared as exact fractions, so every node picks the same leader.
- **A phase move cites a plan.** When someone proposes leaving a plan phase, the coordinator fills
  in the current leader (or refuses with web's message if there is none or it is tied). Integrity
  checks that the cited record is a plan proposal. When the decisions are folded into the
  project's state, a move applies only if the cited plan is approved, of the kind that phase
  needs, and closed before the move closed. Later ratings can't undo a move that already passed.
- **The subtype chosen** by the cited production or operations plan becomes the project's
  subtype from that move on (`Applied.subtype`).
- Value criteria ("How well does this plan satisfy …") wait for project values.

## Consequences
- New entry and link types change the project DNA hash: a new test network (ADR 0029).
- Rating a plan is a write per criterion. That is fine at project scale, and the newest one wins.
- A plan that was the leader when the move was proposed still counts if another plan overtakes it
  before the move closes. web re-checks at apply time; here the cited plan only has to be
  approved, which keeps the fold independent of when ratings arrive.
