# Running without the seed node (cancelling Railway)

Owner, 2026-11-10: "this app needs to function without a seed node as it will potentially get
shut down, it's just there for initial start up really… once we have enough desktop clients our
seed node can get shut down and I can cancel my Railway." This plan lists what has to be built
first. It extends ADR 0036 (every full node keeps the network) and README's M7.

Owner, same day: the seed node stays up as long as possible; this prepares for the day no seed
can host. **Desktop and phone apps are what must keep working.** Browser access is good while a
seed (or a community gateway) is up, and fine to lose after.

## What Railway runs today

| Railway service | What it does | Who depends on it |
|---|---|---|
| `bootstrap` | kitsune2 bootstrap (who is in each network) + iroh relay (traffic between nodes behind NAT) | **everyone**, desktop apps included (`native/src/servers.rs`: the saved one, Railway, then Holochain's public dev servers; ADR 0091) |
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

## Bootstrap and relay on desktop full nodes

Holochain nodes find each other through a **bootstrap server** and reach each other across home
routers through a **relay**. Both are Rust crates already in our dependency tree
(`kitsune2_bootstrap_srv`, `iroh-relay`), so **a desktop full node can run them too**. What
limits it is the internet, not the software:
- **First contact only.** Once a node has met peers, they swap peer addresses in gossip
  (kitsune2 `receive_agent_infos` → peer store), so a running network keeps finding itself
  without any bootstrap. A bootstrap is needed for a new install, or for a node that was offline
  long enough for every address it knew to change.
- **Someone must be reachable from outside.** Most home desktops sit behind a router (NAT, or
  carrier NAT with no public address at all), their address changes, and laptops sleep. A
  bootstrap or relay on such a machine can't be reached by a new install. A desktop can serve
  if it has a public address: a forwarded port (or UPnP), or IPv6.
- **The relay needs a name and a certificate** (iroh's relay serves TLS through ACME), so a
  desktop relay also needs a domain or dynamic-DNS name pointing at it.
- **New installs need to know where to look.** The app ships a list; it should also learn more
  from the directory, where serving desktops announce themselves.

So the design is: **"Help the network" mode on desktop full nodes** that are reachable: the app
runs the bootstrap (and the relay, when it has a name) and announces its address in the
directory. Every app ships a list of known bootstrap URLs, adds the ones it learns from the
directory, and remembers peers it has met. With a handful of such desktops, nothing has to be
paid for. A cheap VPS (Fly.io, Oracle free tier, Hetzner) or a Pi with one forwarded port is the
fallback, and iroh's and Holochain's public development servers are a last resort.

Browser access needs a gateway (the seed-node image). Without one, the network is desktop- and
phone-only, which the owner accepts.

Browser access also needs a gateway. Without one, the network is desktop- and phone-only. A
community member can run a gateway later; it is the same seed-node image.

## Build list, in order

1. **Desktop app works today, tested** (S–M; in progress 2026-11-10: `.github/workflows/desktop.yml`). `native/` was last changed on 2026-10-19 and no
   CI job builds or runs it (only the phone conductor check): build it in CI for
   Linux/Windows/macOS, run a smoke test on Linux (sign in, open a project,
   chat), and fix what broke (the commons, private spaces and `/session/cells` came after it).
2. **Sign in with your words on desktop** (S; ADR 0090 built 2026-11-10, hosted-copy retirement and the sign-out warning ADR 0092, 2026-11-11). Today the desktop app makes its own agent and
   pairs with a hosted account (ADR 0024); importing the phrase is "reserved for M7". Every
   hosted person must be able to move their own key to their desktop (IDENTITY.md's way out).
   Also check that passkey-only people can still see their phrase before the seed goes.
3. **Several bootstrap/relay URLs** (S; built 2026-11-11, ADR 0091). A list in the app and a Settings field to add one;
   the app tries them in order (`native/src/servers.rs`; the first that answers `/health` at
   start). The guide: `docs/RUN-A-BOOTSTRAP.md`.
3b. **"Help the network" mode on desktop** (M; built 2026-11-12, ADR 0094: `native/src/helping.rs`, `announced.rs`, the health coordinator's `announce_server`/`list_servers`; the three-desktop test is still to do): the desktop app runs `kitsune2_bootstrap_srv`
   (and `iroh-relay` when given a domain) when the person turns it on and the node is
   reachable; it announces the URL in the directory; every app reads announced URLs into its
   list. Test: two desktops find each other through a third desktop's bootstrap with Railway's
   bootstrap switched off.
4. **Archivist + index in a crate every full node runs** (L, the biggest piece). Move
   `services/gateway/src/node/{archivist,index,cards,event_time,people,markers,region,spaces}.rs`
   into a crate without the gateway's HTTP layer. The desktop app runs it in-process and the
   UI's `holo/feed.ts` asks the device (Tauri command) instead of the gateway when
   `inNativeApp()`. The gateway keeps using the same crate. Memory: the seed node uses ~650 MB
   for this today; measure on a desktop and keep it optional (ADR 0037's Storage setting).
5. **Geocoding from the device** (S; built 2026-11-12, ADR 0093: `services/geo`, `native/src/places.rs`). In the native app, call Nominatim (with an identifying
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

## Where it stands and what comes next (plan of 19 Nov 2026)

Owner, 19 Nov: "time to work towards seed node not being necessary. Still only need to plan for
this now as not much usage." The FastAPI site is gone, so this is now the main track after the
last parity checks. Nothing here is urgent: build it in this order, one PR at a time.

**Done:** 1 (desktop builds in CI for Linux, Windows and macOS, plus a smoke test), 2 (phrase sign-in and
moving a hosted account), 3 (bootstrap list), 3b (Help the network, except its test), 5
(geocoding on the device). Deactivation's directory part (ADR 0095).

**Phase A: prove what is built (S).** Owner, 19 Nov: test it soon. Railway's bootstrap stays on as long as possible, as a backup, never the only way in. The 3b test, as a scenario rather than three real
machines. Three conductors, Railway's bootstrap off, and the third conductor runs
`kitsune2_bootstrap_srv`. The other two find each other through it, then keep talking
after it stops, using gossiped addresses. This is what makes Railway's `bootstrap` replaceable.

**Phase B: the feed without the gateway (M, step 4).** Owner, 19 Nov: it shouldn't take long. Most of it is moving code that already works.
- B1 Move the gateway's `node/{archivist,index,cards,event_time,people,markers,region,spaces}.rs`
  (about 2,000 lines) into `services/archive`, a crate that depends on `holochain` like
  `services/geo`. Leave out axum and the account tables. The gateway uses the crate unchanged.
  The same tests move with it. The `ci` check proves nothing changed.
- B2 The desktop runs it in-process when Storage is "keep everything" or "share a slice"
  (ADR 0037). Add Tauri commands for projects, people, scopes, paused, markers and region.
  `holo/feed.ts` asks the device when `inNativeApp()`, as places already do (ADR 0093).
  Measure memory on a desktop (the seed uses about 650 MB for this), and keep it optional.
- B3 Decide, as a new ADR, how a **light** node gets a feed (phones, and desktops set to light)
  with no gateway. There are two options:
  - (a) Ask a full node over Holochain: the archivist agent answers a remote call or signal.
    Full nodes are found from the directory's announced servers.
  - (b) Build a small index on the device from the commons' recent time buckets (ADR 0065).

  **Decided (owner, 19 Nov): (a).** Phones ask the desktops that hold the data.

**Phase C: updates and resets without the gateway (M, step 6).**
- C1 Tauri's updater, fed from GitHub Releases. `desktop.yml` already builds the installers,
  so this adds signing keys and a release step. macOS needs the owner's Apple developer
  account (US$99 a year), and Windows wants a code-signing certificate.
- C2 Write an ADR on resetting the networks once real data matters. Today an integrity change is a new
  network, and `release.rs` on the seed swaps coordinator zomes and drops old networks. Options:
  - Group integrity changes into rare planned resets, with an export and import of each person's own chain data.
  - Make zome reads take both the old and the new entry shapes, so most changes stay coordinator-only.

  **Decided (owner, 19 Nov, on Claude's advice): both.**
  - Day to day, reads accept old and new entry shapes, so most changes ship as coordinator updates.
  - When integrity has to change, changes are grouped into a rare, announced reset. Each person's app exports their own data from the old network and writes it into the new one.
  - The ADR is still to write when C2 is built.

**Signing (owner, 19 Nov).** Every update carries Tauri's own update signature: a free key we
hold, checked by every desktop before it installs. It matters even more once peers serve
updates, because any peer could serve a tampered file.

Apple's developer account and a Windows certificate only quiet the operating system's
warning on a first install from a browser. The app's own updates don't get that warning.
So ship without them, with a short note on opening the app the first time, and buy them
when non-technical installs matter.

**Code off GitHub (owner, 19 Nov: use an existing system).** The candidate is Radicle,
which is peer-to-peer git. Desktops can optionally run a Radicle node holding the repo, and the updater fetches releases from
them. A repo's identity document names its *delegates* (keys) and a *threshold*. The canonical
`main` is the commit that the threshold of delegates have pushed.

How merging follows votes:
- **Merge bots are the delegates.** Each bot runs on a different desktop with its own key, for
  example threshold 2 of 3.
- **Each bot decides on its own.** It reads the PR's vote from our network, and if the PR
  passed, it pushes the PR's head to `main`.
- **Fast-forward only.** PRs must be brought up to date with `main` first, so every bot pushes
  the identical commit and the threshold can agree.
- **One bot can't merge alone.** A single compromised or buggy bot can't move `main` by itself.
- **Platform-tagged PRs** also need a seated moderator's sign-off in our network (owner's choice,
  until the bots have proven themselves). The bots wait for it.

Radicle's canonical-reference rules can later hold a second rule per branch if needed. To
check before building:
- Radicle's maturity on Windows.
- Whether a Radicle node can be embedded in the desktop app or has to run beside it.
- How a PR's head on Radicle links to its vote record.

**Software projects pick their repo (owner, 19 Nov).** When a plan makes a software project,
it chooses one of two options:
- **Bot-managed:** the merge bots create a Radicle repo and merge PRs by the project's votes.
- **Linked:** the project enters a repo it already has, and the app links to it.

Goal: in the end, everything runs autonomously.

**Phones keep their own projects (owner, 19 Nov; amends ADR 0037).** A phone keeps a copy of
everything in the projects, chats and spaces it is a member of, and it works offline. When it
needs data, it asks in this order:
1. An always-on desktop.
2. Any other member's phone that is open and newer.

Rules:
- **Private spaces:** these are their own small networks, so a member's phone can hold the whole thing.
- **Public spaces:** these live in the shared commons, so the phone keeps its own copy and answers fellow members directly.
- **Only while the app is open.** Phones hold or serve nothing else and do nothing in the background, which is what the app stores and batteries need.

Android can add "help while charging" later.

**App store checklist.**
- **Both stores:**
  - User-generated content needs reporting (built), moderators (built), blocking a person (check), and terms people accept at sign-up (to add).
  - Account deletion in the app (ADR 0088).
  - An encryption declaration (standard encryption, so exempt, but it must be filed).
  - A privacy label.
- **Google Play:** a foreground service, declared as data sync, if syncing ever runs outside the app.
- **Apple:** the app must work fully for the reviewer, including logging in and finding content, with no desktop of their own. While the seed or a community node is up, that is easy; after the switch-off, there must be at least one always-on desktop. iOS still needs ADR 0020's keystore fix, or ships as a thin client first.

**Phase D: the gauge and the leftovers (S each).**
- D1 Readiness count (step 7): the `health` heartbeat carries the node type and app version, and a
  page counts full nodes seen in the last day and how many hold each public space.
- D2 Deactivation's reader rules (ADR 0088), with the next directory reset.
- D3 Push without the gateway. Desktops already use OS notifications. Android needs its own
  route (the app's notification while syncing, or a push service). Decide this when phones come.
- D4 Android build in CI (step 9). iOS stays blocked upstream (ADR 0020).

- D5 Chat privacy before real users. Owner, 19 Nov: DMs and group chats live only on their
  members' devices.
  - **Already true:** each chat is its own network that only its members join. The archivist and the guest reader never hold chats (`public.rs`).
  - **Gap 1, joining (to fix):** the membrane isn't checked when someone joins (ADR 0043). Anyone who learned a chat's secret seed could join and read it.
  - **Fix for gap 1:** a membrane proof, the invitation, checked at genesis. This is a conversation integrity change, so it means a conversation reset.
  - **Gap 2, hosted accounts:** browser accounts keep their keys and chats on the seed, so the operator could read them. Moving to a desktop or phone ends that.
  - **Later:** invitations sent by remote call when the recipient is online, to hide who invited whom.

**Separate from seedless, before growth:** a second gateway, because `VITE_SP_GATEWAYS` already
takes a list. Hosted agents' memory limits the seed first, well before cost does.

**Rough size:** A about one session. B1 one to two, B2 one to two, B3 one to two. C1 one (plus the owner's accounts), C2 one for the ADR. D one or two each.

## When to switch off

- At least ~5 desktop full nodes online most of the day, in different homes, each holding the
  directory and the commons (step 7's page says so), for a few weeks.
- Every hosted account has moved to a desktop (or its owner accepted losing browser access);
  announce a date and keep the seed node read-only for a month.
- At least two bootstrap/relay copies run by different people (desktops in "Help the
  network" mode or small servers), all in the shipped list or announced in the directory.
- Take a final copy of the seed node's `/data` first; never lose `LAIR_PASSPHRASE` until hosted
  people have moved (their keys are in it).
- Then: turn off `seed-node`, then `ui` (or move the static UI to GitHub Pages for browser
  people with a community gateway), and `bootstrap` last, once the community copies carry it.

## Rough size

Steps 1–3, 5, 7, 8: about a week of sessions in total. Step 4: a week or two. Step 6: a few days
plus the migration decision. Order: 1 → 2 → 3 first (they unblock testing with real desktops),
then 4, then 5–8.
