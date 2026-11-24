# 0116 Only Full members vote, and only they are counted

Status: superseded by 0118 (owner, 2026-11-24: everyone votes)

## Context
ADR 0033 let anyone who was a member vote, then applied a decision only if the recount of
Full members' votes passed. On live, a New founder's lone vote closed their phase proposal as
passed and nothing happened, which read as a bug. New members also counted towards N (the
member snapshot), so a project with a few Full and many New members could need more Yes
votes than it had Full voters.

## Decision
- A New member can't vote (`governance::vote` refuses) and can't propose anything members
  vote on (`propose` refuses, except organizer-controlled events and personal-service
  settings, which apply at once).
- The member snapshot a proposal carries lists only members with Full standing (plus the
  proposer, who is Full). Challenges name only left-out Full members, and `missing` counts
  only them, so nobody is offered a challenge that adds New members.
- The UI says so before anyone taps: a New member's Vote and Phase buttons open a sheet that
  explains Full standing, and says when the project has no Full members yet.

Coordinator only: integrity already accepts a snapshot that lists some of the members, and the
recount (ADR 0033) is unchanged. No reset.

## Consequences
A dishonest node can still challenge with New members' memberships (integrity can't see
standing, which lives in the directory DNA); that only raises N and slows a decision, as any
challenge does. A project with no Full members can't move phase until one joins.
