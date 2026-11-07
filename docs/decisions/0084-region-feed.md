# 0084. The Region feed from the gateway index (M6 phase 4c)

Status: **accepted** (2026-11-07, cloud session; the owner steers less until parity, STATUS
2026-10-25). Plan: [`docs/plans/M6-places.md`](../plans/M6-places.md). Follows ADR 0082, 0083.

## Context
web's feed has a Region scope (`GET /feeds/region`, `get_region_feed` in web-backend's
`feeds/region.py`): public projects, events and help requests with a confirmed physical place
inside the radius, nearest first (ties by `scheduled_at`, else `last_activity_at`), each with
`distance_km` and `is_online: false`; under `all`/`projects` also scheduled, upcoming project
activities at their own place (`project_activity` items, a full personal-service activity left
out); with `include_online`, the most recent online items (place online, or the label
"online"/"remote"/"virtual") appended after the page with `distance_km: null`. The window is
the schedule window (`apply_schedule_window_filter`: outside `all`, only items scheduled in it;
a project's time is its earliest activity). `threads` is always empty. Sorting picks which 300
rows are read before the distance filter.

## Decision
- **`GET /index/region`** (gateway only, no zome change, no reset) with web's parameters
  (`lat`, `lon`, `radius_km`, `sort`, `window`, `filter`, `include_online`, `limit` 1–100,
  `offset`) and the feed's `tzOffset` in place of web's zone name. Its checks and refusal
  codes are the map's (`region_coordinates_out_of_range`, `invalid_region_radius`).
- **Shape:** the app's feed shape, not web's snake case, since the UI already reads
  `/index/projects` cards: `{ items, total, sort, window, filter, limit, offset, radiusKm,
  center, includeOnline }`; a card item is the feed card plus `distanceKm` and `isOnline`; an
  activity item is web's frontend `PublicProjectActivityItem` (`kind: "project-activity"`,
  `href` `/projects/<id>?activity=<id>`, `parentId`, `parentTitle`, `projectMode`, times in
  microseconds as elsewhere in the index).
- **web's rules kept as they are**, including two that differ from the map: a project
  activity counts only at its *own* place (the map falls back to the parent's), and only a
  *personal* service's full activity is left out (the map leaves out both services').
- Cards learn whether their place is online (`ProjectCard.online`, not sent): the charter's
  place, or an event's leading plan's when it names one.

## Consequences
- Pure reads over the cards the archivist already builds (ADR 0083 brought the activities).
- The UI's Region scope, distances on cards and the map panel are phase 5.
