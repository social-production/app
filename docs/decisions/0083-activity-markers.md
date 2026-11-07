# 0083. Activity markers, "Open spots" and time windows on the map (M6 phase 4b)

Status: **accepted** (2026-11-07, cloud session; the owner steers less until parity, STATUS
2026-10-25). Plan: [`docs/plans/M6-places.md`](../plans/M6-places.md). Follows ADR 0082.

## Context
web's `get_map_markers` also pins scheduled activities (event activities under the events
filter, project activities under projects), at the activity's place or, when it has none, its
parent's (`coalesce`); a personal or collective service's full activity gets no pin; a
collective service's project pin reads "Open spots" while an upcoming activity is short of its
minimum; and timed items keep to a calendar window (`today`/`week`/`month`) and
`date_from`/`date_to`. The index read activities only for help requests, and the activities
zome's view did not expose an activity's place (ADR 0078 stored it).

## Decision
- **`ActivityView.location`** (coordinator only, no reset): the activity's `Place`.
- **The archivist reads every re-read space's activities** (it re-reads only new and recently
  written spaces, plus a full pass every `FULL_EVERY` rounds) and keeps them on the card
  (`ProjectCard.activities`, not serialised into feed cards): id, title, times, label, place
  (`Own`, `Parent` or `Elsewhere` for online), and web's commitment counts (people in roles;
  the sum of required counts).
- **Activity markers** in web's shape (`entity_type: "activity"`, `activity_source`,
  `parent_id`, `parent_title`, `href` `/events|projects/<id>?activity=<id>`,
  `committed_count`, `minimum_participants`). Help requests keep their one pin.
- **"Open spots"** on a collective service's pin (web's `_project_pin_subtitle`; personal
  services stay "Accepting requests", ADR 0082).
- **Windows and dates:** `window` with the feed's `tzOffset` (minutes east of UTC, as
  `/index/projects` takes it) in place of web's zone name, and `date_from`/`date_to` as whole
  UTC days (`YYYY-MM-DD`; other forms are refused with web's `invalid_date_filter`). Project
  pins have no time and stay, as on web.

## Consequences
- One more zome call per re-read space each archive round. The Region feed (web's
  distance-sorted feed with project-activity cards) is phase 4c.
