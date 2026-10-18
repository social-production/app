# Governance

Decisions are made by votes inside project cells. There are no admins, and the board only
executes (merges approved PRs, buys approved assets). On P2P there is also no server to declare
"the vote passed", so **every decision is a record that any node can verify**.

## One primitive: Proposal → Vote → Outcome (ADR 0009)

```
Proposal { kind, target, payload, created_at }        authored by a member
   ▲ cites
Vote     { proposal, choice: Yes | No }               one live vote per agent (latest wins)
   ▲ cites
Outcome  { proposal, votes: [ActionHash], result }    authored by ANY member
```

- **kind** names a registered decision type (edit, phase change, update, link, plan, PR merge,
  merge-capability change, repository replacement, service settings, moderation, bot review,
  board membership…). Each kind registers its `Threshold`, population rule, payload
  validation and **apply rule** (what changes once it passes).
- **Outcome validation.** Validators `must_get_valid_record` every cited vote, check each voter was
  eligible, drop superseded votes, recompute the tally with `sp_governance::tally`, and reject
  the Outcome unless the claimed result matches. False Outcomes cannot be written.
- **Apply.** Project state (charter, phase, roles…) is never edited directly. Coordinator reads
  compute current state from the chain of valid Outcomes. History equals the Outcomes.
- **Special kinds** may hand-write their tally where yes/no does not fit (e.g. plan selection with
  value votes and criterion ratings). They still end in a validated Outcome.

## Eligibility (ADR 0008)

A vote counts when the voter:

1. is a **member** of the project (joining is instant and open: ADR 0006), and
2. holds **full standing granted before the proposal was created**.

Rule 2 is platform-wide (from the directory DNA), so brand-new projects can vote at once,
while accounts created after a proposal opens cannot swing it.

## Standing (ADR 0007)

Levels: `New → Full`, or `Paused` (bot review). Only **Full** votes count. (FastAPI's "warm" step
is dropped: M1 plan, docs/plans/M1-identity.md.)

There is no live counting. A level is held through a **`StandingGrant`** that **cites its evidence**:

- **2 vouches** from distinct full-standing agents (each `Vouch` is its own record), or
- **steady activity**: chain 30+ days old with activity on 10 distinct days, provable from the agent's own chain.

Validators fetch exactly the cited records and check them, so every node reaches the same verdict.
Each voucher may vouch **5 times per 30 days**. If someone you vouched for is paused by a
`BotReview` Outcome (3 marks open it; full members decide), you lose the right to vouch for
**90 days**. You keep your own standing.

**Bootstrap.** A named founder list of agent keys (directory DNA property) starts Full so that someone can vouch.

## Quorum and thresholds

- `Threshold::DEFAULT` = 66%, integer basis points, so every validator agrees exactly.
- `required_votes(population)` is ported 1:1 from `web-backend/app/utils/votes.py` and tested against it.

### Open design item (M3): the population N

FastAPI uses "weekly active members". That is a live count no validator can reproduce.
Proposed rule, to be decided before M3:

- **Snapshot at creation.** The proposal cites the membership/standing set it was created against
  (count + Merkle root over member action hashes).
- **Challenge window.** An Outcome takes effect after a window (e.g. 48 h). During it, anyone
  can publish a `Challenge` citing members that were omitted, or invalid voters. A valid
  challenge voids the Outcome, which is then rewritten with the larger population.

This keeps validation deterministic and leaderless while stopping "under-count the population" attacks.

## What the board can and cannot do

Board members hold no legislative power in the data model. Their only special records are
**execution receipts** (e.g. "merged PR #123 per Outcome X", "bought asset per Outcome Y")
that must cite a passed Outcome. Validators reject receipts without one.
