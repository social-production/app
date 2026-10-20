# 0042. Links between projects are a pair of proposals, one in each project

Status: accepted (2026-10-20). See docs/plans/M3-governance.md ("M3e"). Builds on
[0030](./0030-project-clones-and-creator-seed.md) and [0033](./0033-project-governance-checks.md).

## Context
web's Links tab connects a project with related projects and events (web-backend
`detail_links.py`). Any member of one record proposes a link to another, with a reason; the
members of **both** records vote, each side under its own population and the 66% threshold,
and the link is active once both sides pass (rejected once either can't). Active links can be
put to a sever vote the same way, from either side. Votes on links come without the proposer's
Yes.

Here each project is its own network (a clone of the project DNA, ADR 0030). A cell can't read
or validate another network's data, and a member of one project is usually not a member of the
other, so there is no single place to hold the request and its two tallies.

## Decision
- **A request is two proposals.** `ProposalKind::Link(LinkRequest { other, relationship,
  summary, sever, source })` is one project's half. The proposer files it in their project
  (`source: None`), then the app files the mirror in the other project, naming the first as
  its `source`. `other` names the other project by directory listing and network seed.
- **The mirror may come from a non-member.** `Proposal.membership` became optional: only a
  mirrored link half may have none (integrity checks this), and its member snapshot is the
  other project's members, counted and challengeable as any snapshot (ADR 0040).
- **Each half is decided by its own project's rules** (votes, outcome, Full-standing recount),
  with no proposer vote, as on web. Nothing about links is applied to project state.
- **The reader pairs the halves.** `link_sides` reports a project's halves; the UI reads them from
  each linked project and hands them to `list_links`, which pairs them (`sp_governance::links`:
  each names the other as source, same text; a sever's create halves must be paired too; the
  earliest mirror wins) and groups them as web does: active, pending, historical, past votes.
- **Severing** files a `sever: Some(create half)` pair the same way; integrity checks that the
  cited create half is a link to the same project.
- web's duplicate check ("An active link already exists") needs both sides, so the app makes it
  before filing.

## Consequences
- New proposal kind and the optional membership change the project DNA hash: projects were
  reset again (people stay).
- Opening a Links tab clones every linked project's cell on the viewer's node (one more network
  joined per linked project), as visiting those projects would.
- If the mirror write fails, the request shows as waiting for the other project to receive it;
  the proposer can't re-send it yet.
- Anyone can file a mirrored half in any project, as anyone can propose a link on web; it does
  nothing unless that project's members approve it.
- Events have no screens yet, so only project-to-project links exist; event links follow the same
  design when events are ported.
