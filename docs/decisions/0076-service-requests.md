# 0076. Service requests: a request lives in the direct chat with the service's creator; request settings are a proposal

Status: **accepted** (2026-11-06, cloud session; the owner steers less until parity, STATUS
2026-10-25). Parts (a), (b) ([0077](./0077-availability-and-bookings.md)) and (c) ([0089](./0089-collective-service-requests.md)) built. Builds on
[0043](./0043-direct-messages.md), [0056](./0056-activities-and-roles.md),
[0059](./0059-types-evolve-through-plans.md) and [0061](./0061-private-events-and-organizers.md).

## Context
PARITY's "Service requests" item named web's `features/requests` borrowing and delivery frames.
Those are dead code on web: `RequestFrameStack` is imported nowhere, and every frame says
"Frame only". The working feature is web-backend's project service requests
(`projects_service_requests.py`, `projects_service_request_settings.py`,
`projects_service_availability.py`) and web's screens for them:

- **Personal services** (IndividualServicePhaseOne): anyone signed in except the creator sends
  a request (title, details, an optional time); it opens a one-to-one chat with the creator and
  posts `Request "<title>" · Pending … View request: …` there. The creator reviews in the
  Requests sheet (Accept, Decline with an optional reason); the person asking sees "Your
  requests" and may cancel while it is pending; each decision posts a status line into the
  chat and sends `svc-request` notifications. "Only you and each person asking can see their
  request." The creator's Request settings (enable, Scheduled slots / Message requests / both,
  off-schedule requests) apply at once; web's create flow asks for the request mode.
  Availability (one-off slots and weekly hours), slot holds and bookings (an accepted timed
  request becomes an activity with a Provider role, rated in Request history) sit on top.
- **Collective services** (CollectiveServicePhaseFive): request settings change through a
  vote (66%, a reason required), and managers plan a request into an activity with roles.
  On web nobody can review a collective service's requests: `is_manager` is set only for a
  personal service's creator (`helpers.py`), so the review half is unreachable there.
- The right rail's Requests section lists open requests you can review (newest six).

## Decision
- **A request is a record in the direct chat** between the person asking and the creator
  (the conversation DNA, ADR 0043), so only those two hold it, as on web. `ServiceRequest
  { project, service, title, body, starts_at, ends_at, invite }` and `RequestStatus
  { request, status, reason, invite }` under the chat's messages anchor (`Requests`,
  `RequestStatuses`). Only direct chats take them; each cites the writer's invitation.
  Integrity: the person asking cancels, the other person accepts or declines
  (`sp_service::may_decide`); only a decline carries a reason (≤ 500). The first decision its
  writer may make settles the request (`sp_service::replay`): web offers Accept, Decline and
  Cancel only while pending. Validators can't know that the other person is the service's
  creator; readers only show requests for services the viewer created (or sent).
- **The coordinator posts web's lines**: `send_service_request` writes the request and web's
  request message; `decide_service_request` writes the status and web's status line
  (`sp_service::request_message` / `status_message`). `service_requests` lists the chat's
  requests, newest first. The UI finds or starts the direct chat with the creator (with no
  first message) and, for the creator, reads every direct chat.
- **Request settings are a proposal**: `ProposalKind::RequestSettings { enabled, mode,
  off_schedule, reason }`, only in service projects. A personal service's creator writes it
  and its Outcome at once without votes (integrity accepts a vote-less pass only when the
  proposal and the Outcome are by the space's founder in a personal service); in a collective
  service it needs a reason and is voted like any proposal. The newest passed one is the
  settings (`Applied.requests`, `get_project`'s `requests`); with none, web's defaults
  (personal: on, both, off-schedule on; collective: off). The chat can't read the project, so
  the sender's app checks the settings before sending (web's `_ensure_requests_enabled`,
  `domain/serviceRequests.ts` `admits`); a request sent around them still reaches only the
  creator, who may decline it.
- **Rules in `crates/sp_service`** (not `sp_core`, which would change every DNA's hash).
- **Parts:**
  - (a) *this change*: the records above; personal services' owner note, "Not taking
    requests.", Requests / Your requests, Request settings and Request service sheets, the
    chat lines, `svc-request` notifications (category Roles) and the rail's Requests rows.
  - (b) availability: one-off slots and weekly hours (`project_service_availability_rules`),
    the "Add availability" composer, slot holds, requests limited to open slots when
    off-schedule requests are off, and bookings (an accepted timed request becomes an
    activity with a Provider role, the requester's check-in and rating in Request history).
    Needs a project integrity change (rules; an activity naming its request).
  - (c) collective services: the Settings ▾ menu's Request settings vote, Request service,
    and planning a request into an activity. Review waits for the owner (below).
  - The create flow's "Service request mode" step comes with (b), where the mode first
    matters (a new personal service takes both kinds, web's default).

## Consequences
- **Two integrity changes at once: the project network and the chats reset** on merge (test
  data only, owner 2026-10-25). The directory DNA is unchanged.
- Requests are as private as the chat: anyone who learned the chat's seed could read them
  (ADR 0043's caveat). A hosted person's gateway operator can read them, as web's can.
- The creator's lists cost one call per direct chat; fine at test scale. An index of requests
  per service would need the inbox to name chats, which ADR 0043 keeps hidden.
- **Owner review:** who reviews a collective service's requests. web has no way to become a
  manager, so its review is dead; the app could let members (or the leading plan's roles)
  review. Owner, 2026-11-10: "both, people who did the activity and the one who received the
  service" (the rating afterwards, both sides: built in (b)). **Accepting, owner 2026-11-10:
  "Anyone can accept it if they are a member. But if multiple roles are needed obviously it
  doesn't go ahead until they are filled, but the option to sign up would appear on calendar and
  right rail."** So in a collective service any member accepts or declines; an accepted request
  becomes an activity with its roles, which goes ahead only once its roles are filled, and
  members sign up from the calendar and the right rail. Part (c) is unblocked.
