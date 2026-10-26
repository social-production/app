# 0056. Activities and roles in projects, plus the waiting project-chat items (M5b)

Status: **proposed** (2026-10-26). Plan: [`docs/plans/M5-work.md`](../plans/M5-work.md) M5b.
One project-DNA reset carries all of it: every live test project starts again.

## Context
web (web-backend `services/projects/actions.py`, `activity_role_suggestions.py`,
`activity_roles.py`, `activity_history.py`; web `ActivitySchedulingPanel`,
`ActivityAgendaList`, `CollapsibleActivityCard`, `ActivityCreationWizard`):
- In the Activity phase (`phase-5`; `phase-1` for personal services) a **member** creates an
  activity: title (≤200), start and end (end after start, start in the future), online or a
  place (≤160) with an optional meeting link (≤500, online only), a note, an optional linked
  plan/phase, and **roles**: label (≤100), required count (≥1), optional maximum (≥ required).
  Members are told ("prj-activity"). There is no edit or cancel endpoint.
- A member **takes one role per activity** while it hasn't ended, unless the role is full
  (`maximum`); they can leave it until it ends. The role's people are told ("role taken").
  Leaving the project releases their roles in activities that haven't ended.
- A member may **suggest** someone else for a role ("prj-role-suggest"); the latest suggestion
  per role stands; the suggested person may decline (the suggester is told).
- After an activity ends, its **participants rate** it 1–5 with an optional comment; they can
  change or remove it. Ratings and "mark done" feed the activity rail's history.
- Waiting since ADR 0044/0049 for the next project-DNA change: photos and files in project
  chats, and private read marks for project chats (today kept per device).

## Decision (proposed)
- **Rules in a new crate `sp_activity`** (not `sp_core`, which would change every DNA's
  hash): field limits, the role-count rule below, the time locks, rating bounds.
- **`Activity` entry** (project integrity), roles embedded (a role is its index in the list),
  citing the author's `Members` link, with web's linked plan stage as a string id
  (`production:<stage index>`, resolved to a label from the approved plans when read); linked
  from an "activities" anchor. Validation: author is
  a member (cited link is theirs and valid), limits, `ends_at > starts_at`. The Activity-phase
  and "starts in the future" checks stay in the coordinator: validation can't know "now" or
  the current phase without citing the outcome that moved there (as web does it in the API).
  Never updated or deleted, as on web.
- **Taking a role is an `Assignments` link** (activity → the person, tag = role index + their
  `Members` link hash), so leaving deletes it (only its author may; project entries are never
  deleted). Validation: target is the author, member, role index in range, the action's time
  before the activity's `ends_at` (deterministic: the action's own time against a fetched
  record); a delete is checked against the same time.
- **Over-subscription without a server:** two people can take the last place at once, and one
  person can take two roles from two devices. The view (`sp_activity::fill`) orders an
  activity's live assignments by action timestamp (then hash) and counts only each person's
  first, and only the first `maximum` per role; the rest show as not counted and the
  coordinator refuses new ones its view already sees as full. Assignments by people who left
  the project don't count for activities that haven't ended (web's release on leave).
- **`RoleSuggestion` entry** {activity, role, person, membership}, latest per role wins; a
  `SuggestionDecline` by the suggested person clears it. Not self, author a member.
- **`ActivityRating` entry** {activity, stars 1–5 or none, comment ≤2000, assignment}: the
  cited `Assignments` link is the author's own on that activity (taken at some point: a link
  later deleted still counts, validators can't see deletes deterministically) and the action
  time is after `ends_at`; the latest per person counts and "none" removes the rating.
- **Notifications** derived on the device (ADR 0046): new activity, suggested for a role,
  suggestion declined, a role in your activity taken.
- **Ride-alongs in the discussion zome:** `ChatMessage.attachments` + `Chunk` (rules from
  `sp_attachment`, as ADR 0049), and a private `ReadMark` per project chat replacing the
  per-device marks (ADR 0044).
- **UI ported from web:** the Activity phase's scheduling panel, agenda list, collapsible
  activity cards (roles, take/leave, suggest, rate), the creation wizard; feed and profile
  activity cards and the right rail's activity follow once this lands (PARITY).

## Open before building
- Whether "mark done" history (`project_service_history_completions`) is in this batch: it
  belongs to service projects' history; leaning: later, with service requests.
- Event activities reuse this when events land (M5c), if events are project-DNA clones.
