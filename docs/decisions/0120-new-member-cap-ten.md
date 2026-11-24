# 0120 New members start ten things a week

Status: accepted (owner, 2026-11-24); amends the New-member limit in docs/plans/M2-projects.md

## Context
A New account (fewer than two vouches) could start 2 listings in any 7 days, and that covers
proposals, threads, posts and help requests alike. After the everyone-votes change (ADR 0118) it
was the main thing a newcomer bumped into: one thread and one post and the week was used up.
The owner: "lets remove the limit or make it like 10 until they have 2 vouchers".

## Decision
- `sp_governance::projects::NEW_PROJECT_CAP` goes from 2 to 10 in a rolling 7 days. Full
  members (two vouches, or 30 days of steady activity) still have no limit.
- The limit stays rather than going away: it is the one brake on a fresh account (or a script
  making many) flooding the feed, and ten is more than a real newcomer starts in a week.
- The create form still shows the count only when one is left.

## Consequences
The directory integrity zome checks the limit, so this is a new directory DNA and so a
directory reset (and the project DNA, which also builds `sp_governance`, if its hash moves):
every hosted account loses its username, profile, vouches and listings. Founders, demo people
and demo content are put back by script afterwards; real people set their username again and
are vouched again.
