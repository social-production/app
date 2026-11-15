# 0106. Convert: close a project and continue its work as another type

Status: **proposed** (2026-11-15). The owner asked for Convert on the phase sheet (Plan, Phase
and Feedback Screens, 2026-11-14: "Convert must still be built"). **Not built:** it changes
project integrity, so the project network resets when it merges. Owner to confirm the reset
and the open questions below.

## Context

web lets members close a project with the outcome "convert": the project closes and a
successor of another mode or subtype starts (web-backend `services/projects/phases/
conversion.py`). Tags and memberships carry over; plans, signals, roles, requests and
inventory stay on the predecessor; a permanent "Converted to" / "Converted from" link joins
the two and cannot be removed through the Links tab.

Since ADR 0059 a proposal gets its type from its approved plan, so a new idea never needs to
convert. Convert is for work that changes shape later: a built project that becomes a
service that runs it, or a one-off event that becomes a recurring service.

## Decision (proposed)

1. **The phase sheet gains a third kind of choice** under the move on and the returns:
   "Convert to a service" (or "Convert to a project", or "Convert to an event", for the
   types it is not). It opens on any phase after Proposal, in danger red like Close, with
   "Closes this and starts a <type> with the same members and tags. Members vote on it."
2. **One proposal kind**, `Convert { to: mode, subtype }`, beside `Phase` and `Return` in the
   project zome's governance integrity. It passes on the same rule as Close (the phase
   vote's majority). Passing closes the project (phase-7, `close_outcome = convert`).
3. **The successor is a new proposal clone made by whoever filed the conversion**, once it
   passes (the network has no server to make it). It starts in the type's Plan phase with
   the predecessor's title, description, place and tags, and invites the predecessor's
   members (each joins by accepting, as with any invite). Until it exists, the closed page
   says "Waiting for <name> to start the <type>", and any member may start it instead.
4. **Lineage** is two link entries: the predecessor's `ConvertedTo(successor)` written with
   the close, the successor's `ConvertedFrom(predecessor)` in its charter. Both show on the
   Links tab under their own heading, with no sever vote.
5. The closed project's page (Plan, Phase and Feedback Screens, 4) shows "Where the work
   continues" as a row linking the successor.

## Open questions for the owner

- Reset: the project network resets on merge. Land it with another integrity change to
  reset once?
- Should members be moved into the successor automatically (they would be members without
  accepting), or invited (as proposed)?
- May a converted project keep its chat history visible from the successor (a link), or
  should the chat be copied?

## Consequences

New integrity: a project network reset. The scenario `governance` gains a conversion
test; browser test `governance.mjs` gains a convert step.
