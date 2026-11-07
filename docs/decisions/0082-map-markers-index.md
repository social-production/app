# 0082. Map markers from the gateway index (M6 phase 4a)

Status: **accepted** (2026-11-07, cloud session; the owner steers less until parity, STATUS
2026-10-25). Plan: [`docs/plans/M6-places.md`](../plans/M6-places.md).

## Context
web's map asks `GET /feeds/map-markers?lat&lon&radius_km&filter&window&…`
(`app/services/feeds/region.py::get_map_markers`): events with a confirmed physical place,
help requests, projects (productive and collective services always, personal services that
take requests) and scheduled activities, within the radius, timed ones only while upcoming,
soonest first; private items only for their members, and private coordinates never leak.
Places now ride on charters, plans and activities (ADR 0078, 0080). The gateway index
(ADR 0031) already reads each public space's charter and, for events, the leading plan.

## Decision
- **Cards carry a place** (`ProjectCard.place`: latitude, longitude, precision): the
  charter's physical place, or an event's leading approved plan's (web: the plan's location
  row replaces the event's; a plan label without coordinates drops the pin). Online places
  have no pin. Older charters and plans without places still decode (`serde(default)`).
- **`GET /index/map-markers`** on the gateway (`routes/map.rs`, `node/markers.rs`): web's
  parameters (`lat`, `lon`, `radius_km` 1–20 000, default 25; `filter` all / projects /
  events / help_requests; `upcoming_only`, default true; `limit` 1–500, default 200), web's
  refusal codes (`region_coordinates_out_of_range`, `invalid_region_radius`) and web's
  `MapMarkersResponse` shape in snake case. Times are Holochain microseconds, like the rest
  of the index. Distance is `sp_place::distance_km`, rounded to 0.01 km.
- **Privacy by construction:** the index lists public spaces only (private ones are clones
  whose records stay with their members), so web's per-viewer visibility clause has nothing to
  filter. Removed items (ADR 0071) get no pin.
- **Personal services pin as "Accepting requests"**: requests are on by default (ADR 0077) and
  the index doesn't read the setting. **Not in 4a:** activity markers, projects' "Open spots"
  subtitle, calendar windows and date ranges (they need activities in the index), and the
  Region feed; those are phase 4b.

## Consequences
- Gateway only: no DNA change, no reset. The map panel (phase 5) can read markers as soon as
  this deploys; cards in `/index/projects` gain an optional `place` the UI ignores until then.
