# 0118 Everyone votes; a bot finding takes it away

Status: accepted (owner, 2026-11-24); replaces 0116, amends 0007, 0008, 0033, 0074

## Context
New members couldn't vote, propose a vote, mark a bot, vote in bot reviews or volunteer as a
moderator until two Full members vouched for them or they had a month of steady activity. The
owner: "It could be too restrictive for new people if they can't get involved straight away."

## Decision
- Every account votes from its first day, with the same weight. Only a bot review that found
  the account to be a bot (ADR 0074) takes that away, from the time it was found.
- One rule decides it: the directory's `full_standing_before(agent, at)` (the name is kept;
  project cells call it) now answers "not paused at `at`". Everything that asked it follows:
  votes and proposals (`governance::vote`, `propose`), the member snapshot, `missing` and
  challenges, plan ratings, value votes, report votes, and the moderator board.
- The weekly active count (platform quorum, ADR 0095) counts everyone active that week who
  wasn't paused then.
- Anyone votes in bot reviews; everyone's votes count (a review doesn't ask whether the voter
  is a bot, which would read reviews in a loop).
- Marking an account as a bot is the one thing a New account can't do (owner, 2026-11-24:
  "That is the only difference between new and full member"): `set_bot_mark` needs the
  caller's Full proof, and a review counts only marks made by someone Full at the time, so
  three fresh accounts can't open a review of anyone.
- Full and New stay as labels: Full means two full members vouched (or 30 days of steady
  activity), shown on profiles and people lists. Apart from marking they carry no weight.
  `get_standing` gains `votes` (false only when paused).

Coordinator only (directory identity, project governance and discussion). No reset.

## Unchanged, because integrity checks them (changing either is a directory reset)
- Only full members can vouch (a `Vouch` cites the voucher's Full proof).
- New members may start 2 projects a week; full members have no limit (a listing without a
  Full proof is checked against the cap).

## Consequences
An account made after a proposal opens can vote on it, so a crowd of new accounts can swing a
vote until people mark them and a review pauses them; the defence is the bot review, not
standing. Only full members can open a review; new accounts can vote in it.
