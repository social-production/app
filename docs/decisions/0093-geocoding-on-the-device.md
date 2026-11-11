# 0093. Geocoding on the device (SEEDLESS step 5)

Status: **accepted** (2026-11-12, cloud session; the owner steers less until parity, STATUS
2026-10-25). Plan: [`docs/plans/SEEDLESS.md`](../plans/SEEDLESS.md) step 5.

## Context
The place picker's search, "use my location" naming and the IP hint go to a gateway's
`/locations/*` routes (ADR 0079), which call Nominatim and ip-api.com. The desktop app asked
a gateway too, so with no seed node a desktop could not pick a place. The owner wants desktop
and phone to work without the seed node (STATUS 2026-11-10).

## Decision
- **The geocoder moves into its own crate, `services/geo`** (package `geo`): `ADR 0079`'s
  `shape.rs`, `memo.rs` and the provider calls, unchanged except that a refusal's status is a
  plain `u16` (no axum) and the search checks are `geo::check_search`. Not a `crates/sp_*`
  crate: it makes HTTP calls (`reqwest`), and those crates are pure (CODE_RULES).
- **The gateway** keeps its routes, now thin over the crate; same answers and codes.
- **The desktop app** has a `locations(route, query)` command (`native/src/places.rs`) taking
  the gateway route's name and parameters and answering its `{ items }`, or `{ status, error }`
  with web's code. The UI's `locationLookup` (`holo/feed.ts`) calls it when `inNativeApp()`;
  `holo/device.ts` turns a refusal into the same `GatewayError`, so the picker's messages
  don't change. Browsers keep the gateway.
- On the device, every lookup counts under one key (web's 30 a minute, 10 IP hints), and the
  User-Agent says "desktop app" (`Settings::from_env_for`; `GEOCODING_USER_AGENT` still wins).
  The IP hint asks ip-api.com about the address the request comes from (no IP in the URL).

## Consequences
- No DNA change, no network reset. The gateway's behaviour is unchanged.
- A desktop's searches go from its own address to OpenStreetMap's servers, as a browser's go
  through the gateway; Nominatim's per-address policy (about one a second) is held by the
  per-device limit, and the gateway keeps its 60-a-minute cap for everyone it serves.
- Phones (M7) will use the same command once the mobile build runs.
