# 0085. The map panel: maplibre-gl + Carto behind web's adapter (M6 phase 5)

Status: **accepted** (2026-11-08, cloud session; the owner steers less until parity, STATUS
2026-10-25). Plan: [`docs/plans/M6-places.md`](../plans/M6-places.md). Follows ADR 0082–0084.

## Context
web's map (`features/map/MapPanel.svelte`, 1825 lines; `maps/{adapter,geo,maplibre,types}.ts`;
`components/cards/map/MapDiscoveryCard.svelte`; `location/{radius,mapPreferences}.ts`;
`stores/mapFocus.ts`) is a panel the top bar's map button opens over the content (and the
`/map` page): a place picker and radius, a time window (with a custom date range), an entity
filter, "Upcoming only", maplibre-gl 6.9 with Carto's Positron style (fallback
`demotiles.maplibre.org`), pins coloured by kind with stacked activities, a radius circle,
and a list of `MapDiscoveryCard`s beside or under the map. Other pages call `openPlaceOnMap()`
to open it on a place. The feed's Region scope reads `/feeds/region`. The app's top bar has the
map button as a disabled stub; the shell already carries web's map-overlay CSS.

## Decision
- **Port, don't redesign:** `maplibre-gl` at web's version (6.9.0) as a UI dependency,
  loaded only with the panel (dynamic import, like web), web's adapter boundary in
  `ui/src/lib/maps/` (split to the 300-line rule: style and colours, marker elements and
  stacking, the adapter), the panel in `ui/src/lib/features/map/` (split the same way), its
  CSS through `PORTS`. Tiles come straight from Carto, as on web (no proxy; the gateway
  answers markers, `/index/map-markers`, ADR 0082/0083).
- **Shell wiring:** `features/shell` may not import another feature (CODE_RULES), so the
  root layout route hands the panel to the shell (a snippet), and `/map` is a route.
  `openPlaceOnMap` is a store (`lib/stores/mapFocus.ts`, web's shape).
- **Map preferences** (radius, window, filter, upcoming) stay on the device, under web's
  storage key, like the default location (ADR 0081).
- **Native:** the Tauri CSP already allows `https:` in `connect-src`; the worker is bundled
  (`?worker&url`, `'self'`). Check on the first native build; add `worker-src 'self' blob:`
  only if maplibre needs it.
- **Sub-phases**, each its own PR: **5a** the panel, the top bar button, `/map`, markers and
  the discovery list; **5b** `openPlaceOnMap` from project, event and activity pages, the
  feed's Region scope (`/index/region`, ADR 0084) with web's `ProjectActivityCard`, and
  distances on cards.

## 5b (2026-11-08)
- **Region scope** in the feed's scope menu (`features/projects/regionState.svelte.ts`,
  `RegionControls.svelte`): web's radius, place search (sets your default place), device
  location and "Include online list"; it starts from your default place once the connection
  settles. Region rows merge with the personal rows as web's combined feed does; project
  activities render as web's `ProjectActivityCard`. web keeps feed choices in the URL; the
  app keeps them per device (as Home/Global already are) and reads web's
  `?scope=region&lat=&lon=&place=&radius=&window=` link, which the map's "Open as list" makes.
- **Show on map:** components reach the map through a context the shell provides
  (`components/map/placeOnMap.ts`, like the paused badge's); the activity sheet's place and
  the Links tab's Venue open the app's map on the place (by label, as web), and modifier
  clicks still open OpenStreetMap.

## Consequences
- The UI bundle gains maplibre (~800 KB minified) in its own chunk, fetched when the map
  first opens. Browser tests stub tiles (no Carto from the sandbox) and check markers and
  the list.
