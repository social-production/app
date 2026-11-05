# 0071. Reports: web's ⋮ Report, replayed from votes by every reader

Status: **accepted** (2026-11-02), part (a); part (b) added 2026-11-03. Follows [0033](./0033-project-governance-checks.md) (Full
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
- **Audience** (web's `electorate_size_for_target`, `sp_moderation::audience`): the largest
  of the members other than the author, the item's commenters, its signallers and the people
  whose votes on this report count, and never under 3. Popularity = signallers + 2 ×
  commenters. **Deviation:** web sizes threads, posts and help requests from the site's
  weekly active users; a P2P network has no global activity count. Sized only from the people
  involved, a new post's audience is its author alone, and web's "can deletion still be
  reached?" rule dismissed every report at its second vote (found by `reports.mjs`); hence the
  voters and the floor of 3, which keeps web's 3-vote minimum reachable. Fewer votes decide
  than on web's public items. For the owner to review.
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

## Part (b), 2026-11-03: comments and chat messages in spaces
web reports comments (`comment` target: a thread's or post's comments and the chats of
projects, events and help requests) from the comment's ⋮ (`DiscussionComment`) and from the
chat bubble menu (`LiveChatPanel`: Report, then "Vote to remove" / "Vote to keep" while the
report is open). A removed one reads "Removed for spam/serious harm"; a serious-harm one hides
behind "Hidden for serious harm report — reveal to read".
- **No integrity change.** The `Report` entry's target is any action, and its rules (not your
  own item, listed on what it reports, votes as yourself and never on your own item) already
  fit a chat message. Part (b) is coordinator and UI only, so the project network is not reset.
- **Coordinator** (`message_reports.rs`): `report_message` (the message must be this space's;
  one open report per message) and `message_moderation(ids)` (only the reported ones come
  back, so an unreported chat costs one link read per message and nothing more).
  `vote_report` reads the report's target and decides on the space or on the message.
- **Numbers** (web's `electorate_size_for_target` and `load_target_engagement` for comments):
  the audience is the space's (members, commenters, signallers, plus the counted voters, floor
  3); the age is the message's; popularity is its voters (comment votes) plus twice the people
  who replied to it on the chat's newest page.
- **UI:** `DiscussionComment` has web's ⋮ (`ReportControl` with `message`); the comment tree
  carries each comment's moderation (`listComments` reads it with the votes). Project, event
  and help request chats (`ProjectChat`) read the reports when new messages arrive and every
  30 s (`chatModeration.svelte.ts`) and pass `reports` to `ChatPanel`, whose bubble menu gains
  Report and the two votes; the composer is web's ReportComposerModal (`ReportComposer.svelte`,
  shared with the ⋮ menu). A refused vote shows above the composer (web toasts it).
- **Deviation:** web keeps a hidden message's photos visible behind the reveal and a removed
  message's files downloadable; the app hides both until revealed (removed: always).
- **Direct and group chats:** done in [0075](./0075-chat-message-reports.md) (web's
  `message` target and its 1:1 path; a conversation-DNA change).

## Consequences
- **Integrity change in the project DNA** (discussion zome): merging resets the project
  network (test data only).
- Removed items stay on the DHT (nothing can be deleted from other people's chains); every
  reader hides them. Their chats and pages still answer direct zome calls.
- web's side effects of a removal: since 2026-11-05 (UI only) a removed project leaves its
  members' right rail (web ends its memberships, open votes and role sign-ups) and its
  members and role holders get web's `prj-removed` card ("This project was removed after a
  spam report.", no actor, opening Home; Settings' Phase finished switch), timed at the
  report (`domain/removedNotifications.ts`). Nothing is deleted from chains; a direct zome
  call still joins or votes. The author's warm-up penalty
  (`record_content_removal_penalty`) comes with bot reviews/standing.
- A report's numbers use the audience and popularity as read now (web stores them per
  vote), so a growing project can make an old report harder to pass on replay. Small in
  practice: both only move the bar by web's boost steps.
