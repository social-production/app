# 0121: Weekly active is the seven days before now

Status: accepted (owner, 2026-11-25). Directory network reset.

## Context
ADR 0095 counted the people active in the last complete calendar week (Monday to Sunday UTC),
one `Active` link per person per week. On Wednesday 25 November that read "Nobody active last
week" although the network was busy: the morning's directory reset (ADR 0120) had wiped last
week, and this week wasn't counted until Monday. The owner: "it should be last 7 days going
back from that exact moment, so updating every second potentially", and every account counts
except bots (ADR 0118 already).

## Decision
- An `Active` link hangs from its UTC day's path (`active.day.<day>`), at most one per person
  an hour (integrity checks the author's chain for an earlier one within the hour). The app
  records after a meaningful action, at most hourly.
- Weekly active at any moment `at` is the number of distinct people with a link in
  `(at − 7 days, at]`, leaving out links written while their author was found to be a bot
  (`sp_active::active_at`). Readers load the eight day paths the window touches.
- Votes snapshot it when they open, as before (ADR 0033), and the moderator board's quorum at
  any moment is `required_votes` of the count then. The board still settles weekly.
- Platform shows "N active in the last 7 days"; ADR 0095's "last week" wording goes.

## Consequences
- The count is right to the hour, not the second: someone whose only action was at 10:20 is
  counted from 10:20 and stops seven days after their link, which is at most an hour before
  their last action.
- A link can't be backdated, so an offline device's actions count from when it comes back.
- The integrity change is a new directory network: accounts, vouches and moderators start
  again (reseeded with the demo and founders).
