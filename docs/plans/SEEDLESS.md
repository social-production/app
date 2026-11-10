# Running without the seed node (cancelling Railway)

Owner, 2026-11-10: "this app needs to function without a seed node as it will potentially get
shut down, it's just there for initial start up really… once we have enough desktop clients our
seed node can get shut down and I can cancel my Railway." This plan lists what has to be built
first. It extends ADR 0036 (every full node keeps the network) and README's M7.

## What Railway runs today

| Railway service | What it does | Who depends on it |
|---|---|---|
| `bootstrap` | kitsune2 bootstrap (who is in each network) + iroh relay (traffic between nodes behind NAT) | **everyone**, desktop apps included (`native/src/network.rs` has one hard-coded URL) |
| `seed-node` | gateway + embedded conductor, `/data` volume | see below |
| `ui` | the static web UI | browser users |

What the seed node's gateway does, and whether a desktop app needs it:

| Gateway job | Browser users | Desktop app today | Without the seed node |
|---|---|---|---|
| Hosted agents, passkeys, PoW signup, sessions, `/ws` proxy | everything | not used | browsers need *some* gateway; desktop doesn't |
| Archivist: holds every public space in the commons and every listed project | availability | not run | **must run on desktop full nodes** (ADR 0036) |
| Feed index `/index/projects` (feed, Home, search, profiles, scope pages, help/event cards) | yes | **yes: the desktop UI calls the gateway's HTTP index** | **must be built on each desktop** |
| `/index/people`, `/index/scopes`, `/index/paused` | yes | **yes** | same |
| `/index/map-markers`, `/index/region` (map, Region feed) | yes | **yes** | same |
| `/public/read` (signed-out page views) | guests | not used | only matters for browsers |
| `/locations/*` geocoding (Nominatim, ip-api proxy) | yes | **yes** (ADR 0079: "the native app calls a gateway too") | desktop calls Nominatim itself (one person's rate is within its policy) |
| `release.rs`: swaps coordinator zomes on deploy, drops old networks | n/a | not used | desktop needs its own update path |

The rest is already peer to peer: chains, governance, chats, notifications (derived on device,
ADR 0046), calls (signals over the network, public STUN), places, moderation.

## The honest limit: a tiny server stays

Holochain nodes find each other through a **bootstrap server** and reach each other across home
routers through a **relay**. Two desktops that already know each other can keep talking for a
while, but new people can't find the network and most home connections can't connect directly.
So "no seed node" really means **no big seed node**: the 650 MB conductor and the hosted agents
go; a small bootstrap + relay stays. It needs no database, little memory, and anyone can run a
copy:
- a free or ~US$5/month VPS (Fly.io, Oracle free tier, Hetzner), or a Pi at someone's home with
  one forwarded port;
- several copies run by different people, listed in the app (a list, not one URL);
- iroh's public relays and Holochain's public bootstrap exist too, but their operators describe
  them as for development: usable as a last fallback, not as the plan.

Browser access also needs a gateway. Without one, the network is desktop- and phone-only. A
community member can run a gateway later; it is the same seed-node image.

## Build list, in order

1. **Desktop app works today, tested** (S–M). `native/` was last changed on 2026-10-19 and no
   CI job builds or runs it (only the phone conductor check): build it in CI for
   Linux/Windows/macOS, run a smoke test on Linux (sign in, open a project,
   chat), and fix what broke (the commons, private spaces and `/session/cells` came after it).
2. **Sign in with your words on desktop** (S). Today the desktop app makes its own agent and
   pairs with a hosted account (ADR 0024); importing the phrase is "reserved for M7". Every
   hosted person must be able to move their own key to their desktop (IDENTITY.md's way out).
   Also check that passkey-only people can still see their phrase before the seed goes.
3. **Several bootstrap/relay URLs** (S). A list in the app and a Settings field to add one;
   the app tries them in order. Write the "run a bootstrap" guide (one Docker command).
4. **Archivist + index in a crate every full node runs** (L, the biggest piece). Move
   `services/gateway/src/node/{archivist,index,cards,event_time,people,markers,region,spaces}.rs`
   into a crate without the gateway's HTTP layer. The desktop app runs it in-process and the
   UI's `holo/feed.ts` asks the device (Tauri command) instead of the gateway when
   `inNativeApp()`. The gateway keeps using the same crate. Memory: the seed node uses ~650 MB
   for this today; measure on a desktop and keep it optional (ADR 0037's Storage setting).
5. **Geocoding from the device** (S). In the native app, call Nominatim (with an identifying
   User-Agent, 1 req/s) and an IP hint directly from Rust; the browser keeps the gateway.
6. **Updates without the gateway** (M). Signed installers and Tauri's updater fed from GitHub
   Releases (free). A coordinator change ships as an app update. An **integrity change is a new
   network**: after cutover that loses everyone's data, so it needs a migration story (read the
   old cell, write into the new one) or must stop happening. Decide this before real users.
   macOS needs an Apple developer account (US$99/yr) to avoid "unidentified developer" warnings.
7. **A readiness count** (S). Desktop full nodes announce themselves in the directory (the
   `health` zome's heartbeat, with node type and app version); a page shows how many full
   nodes were online in the last day and whether each project/space is held by at least N of
   them. This is the gauge for the switch-off.
8. **Deactivation in the directory** (S–M; owner 2026-11-10, ADR 0088): enforced by the
   directory's own rules, not by a gateway, so it still works with no seed node.
9. **Phones (M7)** stay light nodes that sync through always-on desktops. iOS is blocked
   upstream (ADR 0020); Android works. Not needed for the switch-off, but phones without desktops
   online see stale data.

## When to switch off

- At least ~5 desktop full nodes online most of the day, in different homes, each holding the
  directory and the commons (step 7's page says so), for a few weeks.
- Every hosted account has moved to a desktop (or its owner accepted losing browser access);
  announce a date and keep the seed node read-only for a month.
- At least two bootstrap/relay copies run by different people, both in the shipped list.
- Take a final copy of the seed node's `/data` first; never lose `LAIR_PASSPHRASE` until hosted
  people have moved (their keys are in it).
- Then: turn off `seed-node`, then `ui` (or move the static UI to GitHub Pages for browser
  people with a community gateway), and `bootstrap` last, once the community copies carry it.

## Rough size

Steps 1–3, 5, 7, 8: about a week of sessions in total. Step 4: a week or two. Step 6: a few days
plus the migration decision. Order: 1 → 2 → 3 first (they unblock testing with real desktops),
then 4, then 5–8.
