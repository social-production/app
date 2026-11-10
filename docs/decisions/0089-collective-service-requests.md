# 0089. Collective service requests live in a group chat with the members; any member plans them

Status: **accepted** (2026-11-10, owner: "Anyone can accept it if they are a member. But if
multiple roles are needed obviously it doesn't go ahead until they are filled, but the option
to sign up would appear on calendar and right rail."). Part (c) of
[0076](./0076-service-requests.md); builds on [0077](./0077-availability-and-bookings.md).

## Context
web's CollectiveServicePhaseFive has Request service (title, details, a requested window),
a Requests sheet where reviewers **Plan request** (an activity form: title, place, stage,
roles, note → "Schedule activity") or **Decline**, and Settings ▾ → Request settings, a change
members vote on with a reason. On web nobody can review (`is_manager` is only ever set for a
personal service's creator); the owner chose any member. A personal service's request lives in
the direct chat with its creator (ADR 0076), so only those two hold it. A collective service has
no single person to send it to.

## Decision
- **A collective request lives in a group chat** between the person asking and the service's
  members at the time, titled "Request · <title>". Only they hold it, as on web, and (a)/(b)'s
  records, chat lines, `svc-request` notifications and "Your requests" work unchanged.
  Conversation integrity now takes requests in group chats too; the person asking cancels and
  **anyone else in the chat accepts or declines** (`sp_service::may_decide` already said "the
  other side"). The request view names who decided (`decided_by`), for the notification card.
- **Planning is accepting.** A member's "Plan request" opens the page's activity wizard with
  the request's title, time and details and web's one "Service lead" role; they set the roles it
  needs. The activity is a **booking** naming the request (ADR 0077's `Booking`), written
  first; then the request is accepted in its chat. Project integrity now lets any member of a
  collective service book (any roles; the activity's membership is already checked), while a
  personal service's booking stays its creator's with one Provider role. The coordinator
  refuses the person asking and returns the earlier activity when a request is planned twice.
  A member who sees a pending request that already has its activity accepts it (the chat write
  failed after the project's).
- **It goes ahead once its roles are filled:** the planned activity is an ordinary activity,
  so members sign up for its roles from the calendar and the right rail's Project Activity rows,
  and the activity shows its "N signed up · M needed" until it is full. The requester checks in
  and rates it afterwards, as with a personal booking.
- **Request settings** are web's collective sheet: the change needs a reason and starts a vote
  (`ProposalKind::RequestSettings`, already voted in collective services since (a)). Default:
  off, as on web, until the members vote them on.
- **Request service** is offered to anyone signed in while requests are on and the service has
  members other than the person asking. A collective service lists no slots, so calendar-only
  settings ask for a time but don't check it against listed slots.

## Consequences
- **Conversation and project integrity change: both networks reset** on merge (test data).
- Members who join after a request was sent aren't in its chat and don't see it; the members
  at the time decide it. A service with many members makes a large group chat per request.
- Planning writes to two networks (the project, then the chat); the heal step above covers a
  failure between them.
