# 0074. Bot reviews: marks and votes as links on the account, replayed by every reader

Status: **accepted** (2026-11-05). Follows [0007](./0007-cited-standing.md) (standing),
[0033](./0033-project-governance-checks.md) (Full standing counts) and
[0071](./0071-reports-and-moderation.md) (state replayed from votes). Parity item
"Moderation, rest: bot reviews and the paused badge" (`docs/plans/PARITY.md`).

## Context
web's profile action row has **Bot** next to Vouch (`ProfilePage.svelte`): one mark only
flags the account (after web's "Mark … as a possible bot?" confirm); three marks from full
members open a public **bot review** card on the profile ("Bot review open", the tally, Real
person / Bot for full members). web-backend's `services/bot_reviews.py`:
- 66% "bot" of at least `max(3, required_votes(weekly actives))` counted votes upholds it: the
  account is **paused** (read-only, its votes count for nothing) and whoever vouched for it
  can't vouch for 30 days.
- Cleared early once "real person" votes make a bot verdict impossible, or after 7 days: the
  marks are removed and those markers can't mark the account again for 30 days.
- A paused account can ask for **one re-review** 30 days later.
- Notifications: `bot-review` to the account, `bot-review-vote` to full members who share a
  space with it, `bot-review-result` to markers and voters (category Votes needed).
- web's `PausedBadge` follows author names on cards and comments (`bootstrap.pausedUsernames`),
  and the standing chip opens a Trust sheet (Vouchers / Bot marks tabs).

## Decision
- **Records** (directory integrity, `bot_review.rs`): three link types on the account,
  `BotMarks` (target: the marker, tag bot/clear), `BotReviewVotes` (target: the voter, tag
  bot/real) and `ReReviews` (the account to itself). Each is written as yourself, never on
  yourself (re-reviews only on yourself), and **never deleted**: a newer link replaces an older
  one, so every reader replays the same history. A directory integrity change: merging resets
  the directory network (everyone signs up again; test data only).
- **The state is replayed, not stored** (`crates/sp_bot_review`, pure): marks, votes and
  re-review requests in time order give the open review, the closed ones (`log`), live and
  counted markers, cooldowns and the paused periods. Marks and votes count only from people
  with Full standing at that moment (`full_since`); a review opens at the third counted mark;
  web's 7-day expiry, early clear, 66% share, 30-day cooldown and one re-review are kept.
- **Population** (deviation): web sizes the quorum from the site's weekly active users; a P2P
  network has no global count. Here it is everyone who marked or voted on the review, with
  web's floor of 3 votes. So three full members who mark and vote "bot" can pause an account,
  which on web needs the same three when the week is quiet. For the owner to review.
- **Paused**: `get_standing` reads `Paused`; `full_standing_before(agent, at)` is false while
  a review had the account paused at `at`, so projects' tallies (ADR 0033), reports (ADR 0071)
  and bot reviews stop counting its later votes. Votes cast before the verdict still count
  (web recounts with today's standings; a replayed outcome must not flip later). The
  coordinator refuses a paused account's vouches, marks and review votes; the UI hides
  Message, Vouch and Bot for a paused viewer, as web. Other writes (posts, chat) are not
  refused by integrity: web's `ensure_not_paused` is a server check, and a P2P app can only
  refuse in its coordinators (later, per zome, as screens need it).
- **One stance per pair**, as web's `account_stances`: marking withdraws your vouch; vouching
  writes a clear mark. The vouch ban after an upheld review is a coordinator check.
- **Notifications** (`my_bot_review_notices`, derived on the device as ADR 0046): your own
  review's opening and verdict (`bot-review`) and the reviews you marked or voted on
  (`bot-review-result`, plus web's "A bot review of … is open" to markers). Not ported:
  `bot-review-vote` to everyone sharing a space (it needs every space's member list).
- **Paused badge**: the gateway's people index (refreshed every minute) reads each person's
  standing and serves `GET /index/paused`; the shell provides the list to `PausedBadge`
  (Svelte context, as ADR 0071's report actions) after author names in `ContentMetaRow` and
  `DiscussionComment`. Guests may read `get_bot_review` (gateway allowlist).
- **The author's warm-up penalty** after a removal (web's `record_content_removal_penalty`) is
  not ported: it only delays web's warm level, which the app doesn't have (ADR 0007), and
  production runs with `GOVERNANCE_REMOVAL_PENALTY_HOURS=0` (`.env.production.example`).

## Consequences
- Directory reset at merge. The project and conversation DNAs keep their hashes.
- Each standing read costs one more link read (bot marks on the account); accounts nobody
  marked stop there.
- A paused account's own device still lets it write where only the UI hides the action.
