# 0033. Project governance: what integrity checks, what is checked when applying

Status: accepted (2026-10-19). Refines [0008](./0008-vote-eligibility.md) and [0009](./0009-generic-governance.md); settles the
population question in GOVERNANCE.md for M3a.

## Context
A vote counts when the voter is a member and held Full standing before the proposal (ADR 0008).
Membership lives in the project DNA; standing lives in the directory DNA. Integrity validation can
only read its own DNA, so a project's validators cannot check standing. Three more things a
validator cannot reproduce: "weekly active members" (web's N), whether a vote is someone's latest,
and a count of signal links.

## Decision
- **Integrity (every validator, deterministic):** a proposal's author cites their own `Members`
  link; its population snapshot lists distinct `Members` link actions created before it. A vote
  cites the proposal and the voter's own `Members` link. There is one vote per agent per proposal,
  and votes are final (no update or delete), so an Outcome can't pick a stale one. An Outcome's
  claimed result must equal `sp_governance::tally` over its cited, distinct votes, against the
  snapshot size.
- **Outcomes close, they don't decide alone.** An Outcome could cite a convenient subset of votes,
  so at apply time each node recounts *all* the votes it sees that were cast before the first valid
  Outcome, and applies the proposal only if that recount passes too.
- **Apply time (each node, through its own directory cell):** an Outcome changes project state only if
  every counted voter's Full standing predates the proposal. Nodes that have synced the directory agree.
- **Signal gate (leaving Proposal needs ≥ 66% demand):** checked by the coordinator when the phase
  proposal is made and when it closes, not when applying. Signals keep changing, and a passed phase
  must not flip back later.
- **N** is the proposer's snapshot of members (all members, not only active ones). Under-counting is
  answered by the M3b challenge window. Until then a proposer can only shrink N by leaving out
  members, and those members can see that in the proposal.

## Consequences
- Fake Outcomes and fake tallies are impossible. Ineligible voters are filtered consistently, but at
  read time, so a stale node may briefly show a result others don't.
- People can't change a vote, unlike web. This keeps the latest-vote question out of validation.
- N counts inactive members, so quorum can be harder to reach than on web. Revisit with activity
  proofs if it bites.
