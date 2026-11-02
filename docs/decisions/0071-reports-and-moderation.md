# 0071. Reports: web's ⋮ Report, replayed from votes by every reader

Status: **accepted** (2026-11-02), part (a). Follows [0033](./0033-project-governance-checks.md) (Full
standing counts), [0065](./0065-public-commons-network.md) and
[0070](./0070-threads-and-posts.md). Parity item "Moderation" (`docs/plans/PARITY.md`).

## Context
Every web card and detail header has a ⋮ `ReportControl` (`components/shared/ReportControl`,
`ReportMenu`, `ReportComposerModal`): Copy link and "Report <item>" (reason spam or serious
harm, plus what's wrong); once reported, the menu shows the report, its thresholds and No / Yes
votes, and the trigger reads "Under review" or "Hidden". web-backend
(`services/moderation/*`) keeps one active report per item and advances its state at each vote
(`advance_resolution`): the first counted yes → `under_review`; serious harm can reach `hidden`
(cards blur behind a warning) at a lower quorum; deletion (`removed`) needs a quorum from
`required_votes(audience)` (floors 3, or 5 for serious harm) and a yes share of 66% plus
boosts for the item's age and popularity; once deletion is out of reach the report is
`dismissed`. Removed items leave every list and their pages answer "not found".

## Decision
- **Part (a): reports on spaces** (projects, events, help requests, threads, posts). Comments
  and chat messages (web's `comment`/`message` targets, the 1:1 DM fast path), bot reviews and
  the paused badge are later parts.
- **Records** (discussion integrity zome, next to comment votes): a `Report` entry (target =
  the space's charter hash, reason, description ≤ 2000 characters), a `Reports` link (item →
  report, by its reporter) and `ReportVotes` links (report → the voter as themselves, tag
  yes/no). Integrity refuses reporting or voting on your own item, and refuses deleting
  reports or their links: a changed vote is a newer link, so the history every reader replays
  is the same everywhere.
- **The state is replayed, not stored**: `sp_moderation::replay` runs web's
  `advance_resolution` over the counted votes in time order (each person's latest vote
  counts; votes after `removed`/`dismissed` change nothing), weighing each vote with the
  item's age at that moment. Thresholds are web's `thresholds.py` in whole percentages.
- **Who counts**: votes from people with Full standing at the time of their vote (the
  directory's `full_standing_before`, as proposals, ADR 0033). The coordinator refuses votes
  from others. web counts anyone who can view the item; an open network can't, because new
  accounts are cheap.
- **Audience** (web's `electorate_size_for_target`): the larger of the space's members, its
  distinct commenters and its signallers, at least 3 once more than one person is involved,
  minus the author. Popularity = signallers + 2 × commenters. **Deviation:** web sizes
  threads, posts and help requests from the site's weekly active users; a P2P network has no
  global activity count, so they use the same engaged audience as projects (smaller, so fewer
  votes decide). For the owner to review.
- **Coordinator** (`discussion`): `report_space`, `vote_report` and `moderation` (the state
  and the report to show: a removing report wins, else the earliest open one). One open
  report per item, as web's unique index. Writes mark the space active, so the index re-reads
  it within a minute.
- **Index**: the archivist reads `moderation` with each card; cards carry `moderation`
  (`under_review`/`hidden`) and removed items leave the feed, search and profiles. Guests
  may read `moderation` (gateway allowlist).
- **UI**: `components/moderation/` ports ReportControl (state + composer) and ReportMenu
  (styles via PORTS, `report.css`). Components stay presentational: the shell provides the
  service calls through Svelte context (`reportActions.ts`, `features/shell/reportActionsProvider.ts`).
  Every card has the ⋮ in web's place; `FeedSurface` blurs hidden items behind web's
  "Serious harm report" banner; detail pages show the menu in their header and, once removed,
  "Removed for spam" / "Removed for serious harm" instead of the page (web answers not found).
  web's "Link copied" toast is the menu item's own label (the app has no toasts).

## Consequences
- **Integrity change in the project DNA** (discussion zome): merging resets the project
  network (test data only).
- Removed items stay on the DHT (nothing can be deleted from other people's chains); every
  reader hides them. Their chats and pages still answer direct zome calls.
- web's side effects of a removal are left out for now: ending a removed project's
  memberships, votes and role sign-ups, the `prj-removed` notification, and the author's
  warm-up penalty (`record_content_removal_penalty`). They come with bot reviews/standing.
- A report's numbers use the audience and popularity as read now (web stores them per
  vote), so a growing project can make an old report harder to pass on replay. Small in
  practice: both only move the bar by web's boost steps.
