# 0081. The default location stays on the device (M6 phase 3c)

Status: **accepted** (2026-11-07, cloud session; the owner steers less until parity, STATUS
2026-10-25). Plan: [`docs/plans/M6-places.md`](../plans/M6-places.md).

## Context
web's Settings → Regional sets a default location (search, "Set from network" from the IP
hint, and a device-location opt-in that takes one GPS fix when tapped). web keeps it in the
browser (`location/defaults.ts`, `sp_default_location_<user>`) and mirrors the chosen place to
the account (`default_location_id`, `location/sync.ts`), preferring the local copy when it has
coordinates. Its readers are the Region feed and the map (M6 phases 4–5). ADR 0078 said the
app's copy must be private (profiles are public).

## Decision
- **Keep it in this device's storage**, under web's key shape (`sp_default_location_<agent>`)
  and field names (`domain/defaultLocation.ts`, `stores/defaultLocation.svelte.ts`), like the
  app's appearance and notification settings. No account copy for now.
- **Why not a private entry:** a private settings entry is an integrity change in a DNA the
  account already uses (directory or conversation), so a network reset, for one setting
  whose only readers (Region feed, map) don't exist yet; web itself prefers the device copy.
  When more account-wide settings need to follow a person between devices, they get one
  private `Settings` entry together (its own ADR), and this one moves into it.
- **Port web's three rows** (`features/settings/RegionalSettings.svelte`): the picker
  (physical only), the GPS switch (`services/locations.ts::devicePlace`: one fix, named by
  `/locations/reverse`, else "Current location"; web's error sentences), and "Set from
  network" (`/locations/ip-hint`, web's messages). The gateway's `get` now passes the
  routes' error codes through (`GatewayError.message`), so `ip_location_unavailable` reads as
  web's sentence.

## Consequences
- UI only: no reset. A person sets it again on each device, and signing in elsewhere does not
  bring it along (web's server copy would).
- The device fix is stored with full precision on the device only; anything published from
  it later goes through `placeRecord`, which rounds.
