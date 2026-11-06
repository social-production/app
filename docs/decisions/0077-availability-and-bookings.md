# 0077. Availability and bookings: weekly hours are records, one-off slots and bookings are activities

Status: **accepted** (2026-11-06, cloud session; the owner steers less until parity, STATUS
2026-10-25). Part (b) of [0076](./0076-service-requests.md); builds on
[0056](./0056-activities-and-roles.md).

## Context
web's personal service (IndividualServicePhaseOne, web-backend
`projects_service_availability.py` and `update_service_request_status`) lists availability:
one-off slots (an activity titled "Available" with a "Service lead" role) and weekly hours
(`project_service_availability_rules`: weekday, start, end, time zone, note). web-backend
expands the hours into slots for the next eight weeks in their zone, holds a slot when a timed
request that fits it is accepted, and turns every accepted timed request into a booking: an
activity linked to the request whose one "Provider" role the creator holds. When off-schedule
requests are off, a timed request must fit an open slot. Once a booking has ended, the person
who asked checks in ("Requester") and rates it like a participant, in Request history; the
creator's own slots fall under Self planned history. web's create flow asks for the request
mode (Calendar booking / Direct requests / Calendar + direct).

## Decision
- **Weekly hours are `AvailabilityRule { space, weekday, start, end, zone, note }` records**
  (minutes after midnight in an IANA zone), shown while their `Availability` link from the
  space stands; Remove deletes the link. Integrity: only a personal service's creator writes
  them (`sp_activity::availability::check_rule`: weekday 0–6, times differ, zone ≤ 64, note ≤
  120). The UI expands them (`domain/availability.ts`, `Intl` time zones), as web-backend does.
- **One-off slots are ordinary activities** with web's values ("Available", "Service lead",
  "Availability shared by the service creator."): no new record.
- **A booking is an activity with `booking: Some(Booking { request, requester })`**: the
  request's hash in the direct chat (ADR 0076) and who asked. Integrity: only a personal
  service's creator books, for someone else, with exactly one Provider role for one person.
  The coordinator's `book_request` returns an earlier booking of the same request and has the
  creator take the Provider role. Slot holds aren't records: a slot is taken while a booking
  that lies inside some open slot overlaps it (web's holds), and a booking outside every slot
  is its own taken slot.
- **The requester checks in and rates without a role**: `ActivityRating.assignment` and
  `ActivityCompletion.assignment` become `Option`; `None` is valid only from the booking's
  `requester`. The view adds `completion.requester` / `requester_can_set` and `booking`.
- **Accepting books it**: the creator's app accepts in the chat, then books in the project.
  Two cells can't commit together, so a creator's app also books any accepted timed request it
  finds without a booking (`ServiceRequestsState.heal`).
- **"Off-schedule requests off" is checked by the sender's app** against the slots it sees
  (`fitsOpenSlot`), like ADR 0076's other settings: the chat can't read the project.
- **The create flow's request mode** saves a vote-less RequestSettings right after the charter
  when it isn't web's default "Calendar + direct" (web-backend sets off-schedule only for
  "both").
- **Not ported:** web's Timezone selects (the app has no Display timezone setting yet; times
  are the browser's and weekly hours keep the browser's zone), and rule start/end dates (web's
  form never sets them).

## Consequences
- **Project integrity change: merging resets the project network** (test data only, owner
  2026-10-25). Directory and conversation DNAs are unchanged.
- A booking's title and requester are public in the project, as web shows them in history;
  before it ends others see "Taken", as on web.
- A creator could book a made-up request naming anyone, who could then rate it; the request
  itself stays private, so validators can't check it.
