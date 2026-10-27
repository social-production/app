# 0058. Places and the map: OpenFreeMap tiles, Photon search through gateways, approximate by default (M6)

Status: **accepted** (2026-10-27, owner). To be built in M6; coordinates on listings ride M5d's
directory reset.

## Context
web (web `features/map/MapPanel.svelte`, `LocationPicker`, web-backend `models/locations.py`,
`services/locations/{geocoding,ip_hint}.py`, `feeds/region.py`): MapLibre with Carto basemaps;
place search through the server (Nominatim, rate-limited and cached); places stored with
latitude/longitude, `exact` or `approximate` precision, or online; the Region feed filters by
distance (default 25 km); the viewer's position comes from a default location setting, device
GPS or an IP-based guess. Each of these leans on a central server, and entries on a Holochain
network can never be deleted.

## Decision
- **Map tiles: OpenFreeMap** (free hosted OpenStreetMap vector tiles, no key) with MapLibre, as
  web's map otherwise is.
- **Place search: gateways proxy public Photon** (Komoot's open-source OSM search, suited to
  search-as-you-type), with web's rate limit and cache. Devices never call the provider
  directly, so it never sees people's searches or addresses. Desktop apps use any gateway.
- **No self-hosting now.** Later goal (owner): a whole-world map (e.g. Protomaps tiles) and
  search index served by full nodes, sharded once there are enough of them (ADR 0037).
- **Precision: the creator chooses exact or approximate, approximate by default** (coordinates
  rounded to about 1 km). The place's text label is what the creator types.
- **The viewer's position: device GPS and a manual default location, kept on the device and
  never published.** No IP-based guess.
- **Region feed:** the feed index (`sp_index`, on every full node per ADR 0036) filters listings
  by distance; listings gain coordinates and precision in M5d's directory change, so the
  directory resets once for both.

## Consequences
- Two third parties (OpenFreeMap, Photon) until full nodes serve map and search themselves.
- Exact places are public for good; the default protects people who don't think about it.
