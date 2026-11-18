# 0113. Collective requests stay open until planned; the planners' chat grows with the roles

Status: **accepted** (2026-11-18, UX session; owner's design, 2026-11-18). Amends ADR 0089.

## Context
ADR 0089 put each collective service request in a group chat between the person asking and
every member, where any member could accept or decline. The owner pointed out that this reads
as a private message to everyone, and that one member declining says nothing about whether the
service can meet the request. What they want instead:
- Every member is told that a request is open.
- Only the member who plans it starts talking with the person asking. The people who take the
  planned activity's roles join that conversation, so it grows into the group doing the work.
- No declines. A request stays open until someone plans it. If its time passes first, it is
  marked as not met, and the person asking is told once.

Activities have no discussion of their own, so the conversation has to be a chat.

## Decision
- **The request's record stays where ADR 0089 put it**, in the "Request · <title>" group chat
  with the members at the time. Members need it there to see and plan the request, and moving it
  would change conversation integrity, which resets that network. The app now treats this chat
  as the request's **holder**: Messages hides it, and the request is reached from the service's
  Requests list, the notification card and push (to every member).
- **Planning opens the planners' chat.** "Plan request" writes the booking activity and accepts
  the request, as before. It then starts a group chat between the planner and the person asking,
  titled "<request title>", and posts the request's details as its first line. The chat's seed
  is worked out from the activity's author and the request
  (`<planner>.group.<last 24 characters of the request id>`), in the same shape as any group
  chat's seed. Anyone who can see the activity can therefore find the chat without anything new
  being stored.
- **Roles join the chat.** Only a group's creator can add or remove people, so the planner's
  device does it. Each time it loads the service, it adds anyone who has taken a role on its
  planned activities (`add_member`, then the inbox invite) and removes anyone who has left one.
  The planner and the person asking always stay. Someone who takes a role joins the next time the
  planner's app opens the service. Until then, the activity shows "Chat opens when <planner>
  adds you".
- **No declines for collective requests.** The Decline button and its reason box are removed
  for collective services. A member who can't help just leaves the request alone.
- **Not met.** An open request whose preferred end time has passed, or which has no time and is
  more than 30 days old, reads **Not met**. Each device works this out; nothing is written. The
  person asking sees a single `svc-request` card, "Request not met: <title>", from their own
  derived notifications. They can still cancel an open request at any time.
- **Push:** a new request goes to every member, as today. "Not met" isn't pushed, because no
  device writes it; the person asking sees the card the next time they open the app.

## Consequences
- No network reset: only the coordinator-side reading and UI change.
- Messages hides every holder chat (a group chat whose title starts "Request · "), including
  the ones for old requests. Their records are still reached from the service's Requests list.
- The 30-day cut-off is a guess. It is one constant (`REQUEST_OPEN_DAYS`) if the owner wants
  something different.
