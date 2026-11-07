# M6: places and the map (parity item 7)

Written 2026-11-06 by a cloud session from web (`../web`, 2026-10-18) and web-backend. Phases 1–3
done. Each phase starts with its ADR and ships as its own PR. Port from web only
(STATUS "Owner decisions"); asset management stays out.

## What web has
- **A place** (web-backend `locations` table, `app/services/locations/model.py`):
  `display_label` (≤240, required), `latitude`/`longitude` (null when online), `region`,
  `country` (≤120), `provider_place_id` (≤160), `precision` `exact | approximate`
  (approximate rounds coordinates to 2 decimals, exact to 6), `is_online`. Projects, events,
  event plans, activities and help requests keep their `location_label` and point at a
  `location_id`; the label stays what cards show (`feeds/location_display.py`).
- **Geocoding** (`app/routers/locations.py`): `GET /locations/search?q=`, `/reverse?lat=&lon=`,
  `/ip-hint` (`ip_hint.py`), backed by Nominatim (`geocoding_provider_url`, a User-Agent,
  a 24 h cache, 30 requests a minute per IP).
- **LocationPicker** (`components/shared/LocationPicker.svelte`, 649 lines): modes physical /
  online / TBD, search with suggestions, quick picks (your default location, the project's
  places), precision. Used by the create flows (project, event, help request), the plan and
  activity wizards, ProposeEditSheet, the project header, Settings' Regional, onboarding.
- **Regional settings** (`location/defaults.ts`, `sync.ts`, `geolocation.ts`): a default
  location (typed, device GPS, or the IP hint), stored on the account.
- **Map panel** (`features/map/MapPanel.svelte`, 1825 lines; `maps/maplibre.ts`):
  maplibre-gl 6 with Carto's Positron style (`basemaps.cartocdn.com`, fallback
  `demotiles.maplibre.org`), radius (`location/radius.ts`), time window, entity filter,
  `MapDiscoveryCard`s; `stores/mapFocus.ts` `openPlaceOnMap()` opens it on a place.
- **Markers and the Region feed** (`feeds/region.py`): `GET /feeds/map-markers` (events with
  a confirmed physical place, help requests, scheduled activities, projects with open
  spots; private ones only for their members; never private coordinates) and the Region
  scope of the feed (distance-sorted, with project-activity cards and distances).

## Phases (app)
1. **Place records** (ADR 0078, project reset; done on a branch 2026-11-06): `Place` checked
   by `crates/sp_place`, an optional place next to the charter's, edits', plans' and
   activities' labels. Scenario `places.rs`. The account's default location moves to phase 3
   (a private record: profiles are public).
2. **Geocoding on the gateway** (ADR 0079, 2026-11-07): `/locations/search`, `/reverse`, `/ip-hint` as web's routes,
   proxied to Nominatim with a cache and the per-IP limit (gateway env vars like web's
   `GEOCODING_*`). The native app calls a gateway for these too. Needs the seed node's
   network to reach `nominatim.openstreetmap.org` (Railway: fine).
3. **LocationPicker** in the UI (`components/location/`), ported with web's CSS through
   `PORTS` (ADR 0080). **3a** (2026-11-07): the create flows (project, event + upfront plan,
   help request + its activity). **3b** (2026-11-07): the plan and activity wizards (quick
   picks), the edit sheet. **3c** (2026-11-07, ADR 0081): Settings' Regional, the default
   location kept on the device, the IP hint and device position.
4. **Index** (4a 2026-11-07, ADR 0082: cards carry places, `/index/map-markers` for projects,
   events, help requests; 4b: activities, windows, the Region feed): the gateway index reads places; `/index/map-markers` and the Region feed with
   web's visibility rules (private spaces' places never leave their members).
5. **Map panel**: maplibre-gl + Carto (the UI's CSP/hosts must allow `basemaps.cartocdn.com`),
   the top bar's map button, `openPlaceOnMap`, distances on cards, Region in the scope menu.

## Open questions (decide in the phase's ADR)
- Coordinates on a public DHT are public: web keeps private spaces' coordinates off the map;
  in the app a private space's records stay in its own clone, so only its members hold them.
  Approximate precision is the default, as on web.
- IP hints from the gateway see the gateway's client IP (the browser's); the native app's
  request comes from the device, as on web.
