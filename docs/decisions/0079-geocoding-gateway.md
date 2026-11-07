# 0079. Geocoding on the gateway (M6 phase 2)

Status: **accepted** (2026-11-07, cloud session; the owner steers less until parity, STATUS
2026-10-25). Plan: [`docs/plans/M6-places.md`](../plans/M6-places.md).

## Context
web's LocationPicker searches places, turns device coordinates into a label and offers an
approximate location from the person's IP. web-backend answers these at
`GET /locations/search?q=&limit=&country_codes=&viewbox=`, `/locations/reverse?lat=&lon=` and
`/locations/ip-hint` (`app/routers/locations.py`), proxying Nominatim and ip-api.com with a
24 h Redis cache and 30 requests a minute per IP (10 IP hints). A browser can't call
Nominatim directly in a way its usage policy accepts (an identifying User-Agent, one
server's polite rate), and the P2P network has no server of its own besides the gateways.

## Decision
- **The gateway serves web's three routes**, same parameters, same `{ items: [Location] }`
  answers in web's snake-case shape, and web's error codes as the gateway's `error`
  (`geocoding_rate_limit_exceeded`, `geocoding_provider_unavailable`,
  `ip_location_unavailable`, ...), so phase 3 ports web's driver and picker unchanged. Public,
  as on web. The native app calls a gateway for these too.
- **Module `services/gateway/src/geo/`**: `shape.rs` (web's Nominatim/ip-api normalisation and
  checks, using `sp_place`'s limits), `memo.rs` (in-memory TTL cache and fixed-window limits
  in place of web's Redis keys; a restart empties them), `mod.rs` (the provider calls and
  web's status mapping). HTTP through `reqwest` 0.12 with rustls, already in the tree via
  Holochain.
- **Settings** are web's names upper-cased: `GEOCODING_PROVIDER_URL` (Nominatim),
  `GEOCODING_PROVIDER_API_KEY`, `GEOCODING_USER_AGENT`, `GEOCODING_CACHE_TTL_SECONDS` (86400),
  `GEOCODING_RATE_LIMIT` (30), `GEOCODING_RATE_LIMIT_WINDOW_SECONDS` (60),
  `GEOCODING_TIMEOUT_SECONDS` (8), plus `IP_HINT_URL` (`http://ip-api.com/json/`).
- **One addition to web:** at most 60 provider calls a minute per gateway whoever asks
  (cache hits don't count), answered as `geocoding_provider_rate_limited`; Nominatim's policy
  is about one a second per server, and a gateway is shared by everyone it hosts.
- Results keep web's six decimals and say `approximate`, as web's do; the picker rounds to
  the chosen precision when it saves a `Place` (web's `create_location` does the same), so
  `sp_place`'s check holds.

## Consequences
- No DNA change, no network reset: a gateway-only deploy.
- The seed node reaches `nominatim.openstreetmap.org` and `ip-api.com` (Railway allows any
  host). Searches leave the network to OpenStreetMap's servers, as they do on web; IP hints
  only on the person's explicit ask (Settings' Regional), as on web.
- Several gateways each keep their own cache and limits; that is fine at this size.
