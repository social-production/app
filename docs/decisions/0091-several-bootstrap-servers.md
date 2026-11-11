# 0091. The desktop app tries several bootstrap/relay servers and boots with the first that answers

Status: **accepted** (2026-11-11, cloud session; SEEDLESS step 3). Builds on
[0036](./0036-every-full-node-keeps-the-network.md)'s "nothing depends on one machine".

## Context
A Holochain node finds others through one **bootstrap** server and reaches them across home
routers through one **relay** (`NetworkConfig.bootstrap_url` / `relay_url`; the conductor takes
one of each). The desktop app had one hard-coded URL, Railway's. If Railway goes (the owner
wants to be able to cancel it, docs/plans/SEEDLESS.md), every new desktop install, and every
desktop whose known peers' addresses have all changed, finds nobody. `tauri-plugin-hc` already
saves a person's own bootstrap/relay (`set_user_network_config`, which restarts the app) but
leaves applying it to the app.

## Decision
- **An ordered list, chosen at start** (`native/src/servers.rs`): the server saved in Settings,
  then the shipped list. The app asks every bootstrap's `GET /health` at once (4 s timeout,
  blocking `reqwest`, already in the tree) and boots with the first, in order, that answered.
  If none answers it boots with the first, so the node keeps retrying it. `SP_BOOTSTRAP_URL`
  replaces the list (local dev, CI).
- **Shipped list:** Railway's bootstrap + relay, then Holochain's public development bootstrap
  (`dev-test-bootstrap2.holochain.org`) with n0's public iroh relay as the last resort.
  Community copies go in between as people run them (docs/RUN-A-BOOTSTRAP.md, one image).
- **The choice is made once per start, not switched while running.** Peers that have met keep
  swapping addresses in gossip, so a running network does not need its bootstrap; switching
  live would mean restarting the conductor. A dead server is noticed at the next start.
- **Settings → Network** (desktop only): shows the server in use, saves one address (bootstrap
  and relay at the same `https://` origin, as our image serves both) and restarts; "Use the
  built-in list" deletes the saved file and restarts (`reset_network_servers`). The window holds
  `hc:allow-set-user-network-config` (the plugin leaves it out of `hc:default`); the app has one
  window and it loads only the bundled UI, so no other page gets it.
- The seed node and browsers are unchanged: the gateway still uses Railway's bootstrap
  (its `BOOTSTRAP_URL` variable on Railway).

## Consequences
- Two desktops fall back to the same next server only if they see the same servers down. A
  desktop on server B and one on server A meet only through a peer that knows both (gossip
  carries peer infos), so the list order must be the same everywhere: ship one list, add
  community servers in the same order for everyone.
- Holochain's development servers are for testing and may be wiped or rate-limited: fine as a
  last resort, not as the plan. SEEDLESS step 3b (desktops in "Help the network" mode
  announcing their bootstrap in the directory) replaces them.
- Not tested end to end with Railway switched off yet (needs two desktops and a dead first
  server; SEEDLESS 3b's test covers it). The ordering is unit-tested; CI's desktop smoke test
  boots through the list with Railway first.
