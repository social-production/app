# 0078. Places: an optional `Place` next to each location label (M6 phase 1)

Status: **accepted** (2026-11-06, cloud session; the owner steers less until parity, STATUS
2026-10-25). Plan: [`docs/plans/M6-places.md`](../plans/M6-places.md).

## Context
web-backend keeps a `locations` row (label, latitude/longitude unless online, region, country,
the geocoder's place id, precision `exact` | `approximate`) and points projects, event plans,
activities, help requests and edit requests at it, while each keeps its own
`location_label` for cards. The map, the Region feed and distances (M6 phases 4–5) need
coordinates; the app has labels only.

## Decision
- **`project_integrity::Place { label, latitude, longitude, region, country, provider_id,
  precision, online }`**, checked by the new pure crate `sp_place` (web's limits: label ≤ 240,
  region/country ≤ 120, provider id ≤ 160; coordinates in range and already rounded to the
  precision, 2 decimals approximate, 6 exact; an online place has no coordinates, a physical
  one both). A place's `label` must equal the record's own label, so cards and the map agree.
- **Where:** `Charter.place` (beside `location_label`; help requests are charters too),
  `ProposalKind::Edit.place` (beside `location`), `Plan.place` (beside `location`; event and
  other plans), `Activity.location` (beside `place`; online only when the activity is). All
  are `#[serde(default)]` options, so callers that send only labels keep working.
  `Edit.place` and `Plan.place` are boxed (clippy's enum-size rule; same encoding).
- **Edits:** a passed edit that sets `location` also sets the place it carried, or clears it
  (`Applied.place`; a typed label without a picked place has none, as on web).
- **Bookings** (ADR 0077) copy the service's own place when it is the label they use.
- **Not here:** the account's default location (web's `default_location_id`, Regional
  settings) comes with phase 3 as a private record, since profiles are public. Directory
  listings and the index learn places in phase 4.

## Consequences
- **Project integrity change: merging resets the project network** (test data only, owner
  2026-10-25). Directory and conversation DNAs are unchanged; `sp_place` is not in `sp_core`.
- Coordinates in a public space are public, as web shows them on its map; approximate (about
  1 km) is web's default, and private spaces keep theirs inside their own clone.
- The UI sends no places until the LocationPicker lands (phase 3).
