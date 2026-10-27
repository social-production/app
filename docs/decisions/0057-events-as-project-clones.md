# 0057. Events are project-DNA clones with mode "event" (M5c)

Status: **accepted** (2026-10-27). Plan: [`docs/plans/M5-work.md`](../plans/M5-work.md) M5c.
Decides DOMAIN_MODEL open question 1 for events; help requests get their own ADR after events.
**Resets the project and directory networks** (everyone signs up again on the test network).

## Context
web (web-backend `models/events.py`, `services/events/**`, `events_plans.py`,
`plan_criteria.py`, `utils/plan_schedule.py`; web `features/events/**`,
`features/create/event/**`):
- An event has the same machinery as a project: members join and leave, demand/opposition
  signals (66% of all signals to leave Proposal), edit/update/phase votes with a reason,
  plans voted and rated (shared rubric + "Is the timing and schedule realistic?" + "Is the
  location appropriate and accessible?" + the event's values), values with importance votes,
  activities with roles, suggestions, check-ins and ratings, a chat, history and links. Its
  detail page has the same tabs as a project's.
- Phases are `proposal` → `event-plan` → `activity` → `closed`; a return goes to any earlier
  phase. Leaving Event Plan needs a winning plan (web's UI gate).
- An **event plan** adds a schedule to web's plan fields: `mode` any-day | date | range,
  start/end date, start/finish time, stored in UTC (times typed in the author's display time
  zone), and a required place (≤160). The event's time and place follow its leading plan
  (`_sync_event_schedule_from_leading_plan`).
- **Audience:** public events are always collaborative. Private events (`invite_only`,
  `private_community`) may be **organizer-controlled**: the creator and co-organizers decide
  plans and phases without votes.

## Decision
1. **An event is a clone of the project DNA whose charter `mode` is `"event"`** (subtype
   `standard`). No new DNA: membership, signals, proposals, plans, values, activities, chat
   and links all work as for projects (ADR 0030 clones, ADR 0033 governance).
2. **Phase ids are the app's** (`phase-1` Proposal, `phase-2` Event Plan, `phase-5`
   Activity, `phase-7` Closed). web's ids (`proposal`, `event-plan`, …) appear only in
   ported UI code and are mapped there. So integrity's phase check, the activities zome's
   Activity-phase check, notifications and the index need no event branch.
3. **Rules in `sp_governance` as mode `"event"`:** stage labels, visible phases, next phase,
   returns (any earlier visible phase), `PlanKind::Event` in phase 2 (required to leave it,
   like a production plan) and web's event rubric. `sp_governance` is also compiled into the
   directory, whose hash changes in this batch anyway (4).
4. **Directory listings take any kind:** a listing's `mode` is either a project mode (checked
   as before) or another lowercase kind (`[a-z][a-z-]{0,23}`, e.g. `event`, later
   `help-request`, `community`, `channel`) with shape checks only (title, place, phase, tags,
   subtype ≤40). The clone's own DNA holds the real rules, so help requests (M5c) and
   communities/channels (M5d) can be listed without another directory reset.
5. **Event plans carry `schedule: Option<EventSchedule>`** (project integrity), checked by a
   new crate `sp_event` (dates `YYYY-MM-DD`, times `HH:MM`, a range ends on or after its
   start, a date needs a start date) and required for `PlanKind::Event` along with a place.
   Times are stored in UTC as web does; the device converts from and to the display time zone.
   The event's time and place are read from its leading approved plan (no extra entry).
6. **The charter** accepts mode `"event"` through `sp_event`. `sp_core` is untouched, so
   the conversation DNA keeps its hash (old chats are still unreachable after the directory
   reset, because people sign up again with new agents, as after ADR 0054).
7. **First part: public, collaborative events.** Private events (a secret seed as for direct
   chats, ADR 0043, and invites through the inbox) and organizer-controlled governance with
   co-organizers come in a later part with their own project reset; `private_community`
   waits for communities (M5d).
8. **UI ported from web:** the create-event flow (public path), event cards in the feed and
   on profiles, the event page (overview header, lifecycle panel with Proposal, Event Plan
   with the event plan wizard and schedule, Activity limited to the plan's days, Closed),
   and `/events/<id>` routes; project-only parts (subtype, distribution) are hidden.

## Consequences
- One more reset of the project and directory networks on the test network: everyone signs
  up again; projects, follows and chats from before are unreachable (as ADR 0054 did).
- Later kinds (help requests, communities, channels) can be listed without a directory reset.
- Events inherit project fixes for free, and project-only rules must check `mode`.
