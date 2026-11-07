# 0080. The LocationPicker in the UI (M6 phase 3a)

Status: **accepted** (2026-11-07, cloud session; the owner steers less until parity, STATUS
2026-10-25). Plan: [`docs/plans/M6-places.md`](../plans/M6-places.md).

## Context
web's `components/shared/LocationPicker.svelte` searches places as you type (web-backend's
`/locations/search`), offers "Use “…” as the place name" for a typed name, quick picks and
online/TBD chips. On a pick it saves a `locations` row (`POST /locations`, coordinates rounded
to the precision by `create_location`) and the record points at its id. The app has the
place records (ADR 0078) and the gateway routes (ADR 0079); its create flows still had a text
field.

## Decision
- **Port web's picker** as `ui/src/lib/components/location/LocationPicker.svelte` (Svelte 5
  runes), styles through `PORTS` (`styles/location.css`). Two fixes for the port live in
  `additions.css`: the portaled list keeps a `.location-picker` wrapper with
  `display: contents` (web's scoped class reached the list moved to `<body>`; the port's
  prefix does not), and web's `.compact input` (it matched the root's own class).
- **No `locations` rows.** web's `persistOnSelect` and `locationId` have no counterpart: the
  app keeps the place on the record (`Charter.place`, `Plan.place`, `Activity.location`), so
  `domain/places.ts::placeRecord` does what `create_location` did (online place; physical
  only with coordinates, rounded to 2 or 6 decimals; a typed name has no place, as on web).
- **The search is a prop.** Components may not call services (CODE_RULES), so features
  pass `services/locations.ts::searchPlaces` and `searchFailure` (web's two sentences: rate
  limit vs. "Could not search places right now."). The service calls the home gateway's
  `/locations/*` (`holo/feed.ts::locationLookup`), the native app included.
- **Labels are cut to 160 characters** (the charter's, plan's and activity's label limit in
  `sp_core`/`sp_activity`; Nominatim's labels go to 240): changing `sp_core` would reset every
  network, and web's own `location_label` columns are what cards show anyway.
- **3a wires the create flows** (project, event with its upfront plan, help request with its
  activity) through `LocationIntent`. 3b: the plan and activity wizards, ProposeEditSheet and
  the project header. 3c: Settings' Regional default location (a private entry; profiles
  are public) and the IP hint and device position.

## Consequences
- UI only: no DNA or gateway change, no network reset.
- Picked places now land on public records with approximate coordinates (about 1 km), as on
  web; phase 4 indexes them for the map.
