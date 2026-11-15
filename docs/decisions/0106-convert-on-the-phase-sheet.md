# 0106. Convert: close a project and continue its work as a new proposal

Status: **accepted** (2026-11-15). The owner asked for Convert on the phase sheet (Plan, Phase
and Feedback Screens, 2026-11-14) and, on 2026-11-15, approved the project network reset
("reset is fine, it is all mock data, do what you think is best").

## Context
web lets members close a project with the outcome "convert": it closes and a successor of
another mode starts, with the tags and memberships carried over and a permanent "Converted
to" / "Converted from" link (web-backend `services/projects/phases/conversion.py`).

Since ADR 0059 a public project starts as a typeless proposal and its approved plan gives it
its type. So the successor here is a new **proposal**, and its plan decides whether it
becomes a project, a service or an event. No target type is chosen when converting.

## Decision
1. **A new proposal kind, `Convert { reason }`** (project integrity). It is offered on the
   phase sheet as "Convert to a new proposal" (in danger red) in any phase after Proposal
   and before Closed, for projects and services, not events. One phase vote at a time, a
   reason, and the proposer's own Yes, as for a phase move. Channels and communities don't
   convert.
2. **Passing it closes the project** (the fold moves it to `phase-7` and sets
   `Applied.converted`).
3. **The new proposal is started by a member** from the closed project's Take part ("Start
   the new proposal"): the create flow opens at `/create/project?continues=<id>` with the
   title, description and tags filled in. Its members join it the usual way (they are not
   moved: each person joins their own copy of a network, and the feed and the closed page
   lead them there).
4. **Lineage** is one link type, `Lineage`, from a space to itself with the tag `[0]`
   (continues from) or `[1]` (continued by) + the other project's id. Whoever starts the new
   proposal writes both. The project view returns `lineage { converted, continues,
   continued_by }`. The closed page shows "Where the work continues"; the new proposal's
   Details shows "Continues the work of".
5. The chat stays on the converted project and is reachable from it; nothing is copied.

## Consequences
A project network reset. Scenario `convert` covers the vote, the close and the lineage.
