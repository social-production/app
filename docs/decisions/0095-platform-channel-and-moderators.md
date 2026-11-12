# 0095. The Platform channel and its moderators

Status: **proposed** (2026-11-12). Parity items "The Platform channel" and "Moderators"
(`docs/plans/PARITY.md`; owner 2026-11-12: "we also don't have moderators"). Follows
[0068](./0068-channels-and-communities.md) (scopes), [0033](./0033-project-governance-checks.md)
(populations), [0074](./0074-bot-reviews.md) (votes on an account, replayed) and
[0087](./0087-software-votes-elsewhere.md) (merge capability). Pairs with
[0088](./0088-deactivation-in-the-directory.md): both are directory resets, so they ship together.

## Context
web's `/platform` is the `platform` channel's scope page (`DirectoryFeedLayout`): the
description "The channel for the whole network. Follow it to see platform-wide projects,
events, and governance.", Follow, and the projects and events tagged with it. web-backend
(`services/platform.py`) creates the channel on demand, with no creator.
- **Platform-tagged projects** (`is_platform_tagged`) open governance votes to any signed-in
  user, and their population is the site's weekly active users (`gates.py`,
  `resolve_project_vote_population`). A personal service can't use the tag.
- **Moderators** (web's platform board, `services/board.py`, `features/platform/board/*`):
  anyone may volunteer as a candidate; established voters vote yes/no on each candidate or
  moderator (votes persist). A candidate with at least `required_votes(weekly actives)` votes
  and 66% yes becomes a moderator. A moderator whose approval falls below 66% is removed at
  once; one whose vote count falls below quorum gets a one-week grace period, then is removed.
  "How moderator roles work": moderators execute platform outcomes only after users approve
  them; they don't gate creation or decide direction. Their only power in web-backend: the
  merge capability on platform-tagged software projects (`merge_capability.py`,
  "platform-board"). The header's Moderators button and the left rail's "Moderators" link
  open the board.

## Decision
- **The Platform channel is a fixed scope, not a created one.** `sp_scope` accepts a `platform`
  channel record only with web's exact name and description and no manager; readers take the
  earliest such record (any agent may write it; the UI writes it the first time someone
  follows). Follow, its feed and Search work as for other channels; it has no Settings or
  vote sheets (web's has none). `stewardship` stays reserved. Project integrity change.
- **Platform-tagged projects** (`#platform` among the tags): any Full member may vote on their
  plans, phase moves, edits and updates, not only members. Population (deviation, as ADR 0074):
  the project's member snapshot plus everyone who has voted on that proposal, with web's
  floor (until the weekly active count lands; then that count). A personal service can't carry the tag (create flow and integrity). Project integrity
  change (`sp_governance`'s voter check takes the platform flag).
- **Moderator records** (directory integrity, `board.rs`): `BoardCandidacy` (the account to
  itself, tag volunteer / withdraw) and `BoardVotes` (on the candidate, target the voter, tag
  yes / no / neutral). Written as yourself, never deleted; a newer link replaces an older one.
- **Standing is replayed** (`crates/sp_board`, pure, unit-tested against web-backend's rules):
  candidacies and votes in time order give each person's state (candidate / moderator /
  grace / removed), checked at each act and each week's start, so every reader agrees
  whenever it reads. Votes count only from people with Full standing at the time
  (`full_standing_before`, ADR 0074). Quorum: `required_votes(weekly active)` as web, from
  the participation count below. web's 66% share and 7-day grace are kept.
- **Weekly active, counted from signed records** (owner 2026-11-12: "a better count for quorum
  based on active participation"): the first meaningful action of each ISO week (a vote, post,
  comment, role taken) writes one `Active` link from path `active.<week>` to the author in the
  directory, signed by them, one per person per week, never deleted. Weekly active = distinct
  Full members with a link for the last complete week (`crates/sp_active`: weeks start
  Monday 00:00 UTC). Any reader can count it; full nodes
  (SEEDLESS step 4) hold the whole set and answer fastest. A proposal snapshots the count when
  it opens (ADR 0033), and ADR 0040's challenge window covers an under-count. This replaces the
  "voters on this item, floor 3" deviations in ADR 0071 (reports) and 0074 (bot reviews), and
  sizes platform-tagged projects' populations.
- **Powers:** moderators get the merge capability on platform-tagged software projects (ADR
  0087's capability check reads `sp_board`). Nothing else, as on web.
- **UI:** `/platform` becomes web's scope page for the channel plus `PlatformBoardPanel`
  (moderators, candidates, Volunteer / Withdraw, yes / no, the explainer); the left rail's
  Moderators link and the header's Moderators button. Ported styles via PORTS.
- **Resets:** directory (board records, and ADR 0088's deactivation in the same PR) and
  project (the platform scope and voter rule). Test data only; everyone signs up again.

## Consequences
- Each Full member writes one small link a week; it shows that they were active that week
  (web knows the same). Offline devices write theirs on next start, so a week's count can
  grow for a while after it ends: proposals snapshot the last complete week.
- "Execution receipts" (GOVERNANCE.md) stay future work; web has none either.
- Asset management (the board's asset receipts, `?board=1`'s asset frames) stays out of parity.
