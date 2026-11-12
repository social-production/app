# Parity with web (the FastAPI site)

What the P2P app still lacks or does differently from web, found by comparing the two at 390px
(live web: https://web-production-885ec.up.railway.app, read-only; app: a local seed node or the
live UI). Port each item from web's own components; never invent (STATUS "Owner decisions").
Tick items off as they ship, with the PR number. Add findings as you compare more screens.

## What's left for full parity (summary, 2026-11-05, after ADR 0075)
The open boxes below, grouped by size. Asset management is not part of parity (owner,
2026-10-31). Order is the cloud session's (STATUS "Next").
1. ~~Moderation, rest~~ (done 2026-11-05: reports in direct and group chats with web's 1:1
   path, ADR 0075, a conversation reset). (Bot reviews and the
   paused badge: done 2026-11-05, ADR 0074; the warm-up penalty is off on web, not parity.)
   (A removed project leaving the
   rail and its `prj-removed` card: done 2026-11-05.) (The blur on hidden detail pages: done
   2026-11-03.)
2. ~~History tab~~ (done 2026-11-03). Software decisions joined it 2026-11-09 (ADR 0087);
   link create/sever votes 2026-11-10 ("Link decision" / "Sever link decision", web's sheet,
   cast on this project's half; the status is the whole request's, as web-backend's).
3. ~~"Plan became leading"~~ (done 2026-11-03, ADR 0072). Notification kinds web has that
   the app still lacks: `bot-review-vote` (bot reviews' other cards and `prj-removed`: done
   2026-11-05).
   (Project/event pages' "Share +" with `prj-share`/`evt-share`, "Create post", and the
   `thread-share`/`post-share` cards: done 2026-11-04.)
4. ~~Posts and threads, later parts~~ (done 2026-11-04: @mentions in the comment composer,
   Posts in Search, links in post bodies, your own posts on Home and "Following + popular".
   The scope page's create menu is not parity: web's `ScopeCreateMenu` is imported nowhere.)
5. ~~Events, later parts~~ (done: activities limited to the plan's days and the event card's
   time 2026-11-04/05; Remove organizer 2026-11-05, ADR 0073, coordinator only).
6. ~~Service requests~~ (ADR 0076): parts (a) (PR #107) and (b) (ADR 0077: availability,
   slots, bookings, Request history, the create flow's request mode) done 2026-11-06; (c)
   collective services (ADR 0089: group-chat requests any member plans or declines, the
   settings vote) 2026-11-10. web's borrowing/delivery frames are dead code
   (`RequestFrameStack` is imported nowhere): not parity.
7. **Map + places (M6):** done with phase 5b (2026-11-08, `docs/plans/M6-places.md`, ADR
   0078–0085): places on records, geocoding, the LocationPicker, Regional settings, map
   markers, the map panel, the Region scope with project-activity cards, show on map.
8. ~~Software projects~~ (ADR 0086, 0087, `docs/plans/SOFTWARE.md`): the panel, wizard and
   Repository ▾ (phases a + b, 2026-11-08); the rail's rows, Take part's merge capability and
   repository votes, History / Past activity cards and `pr-approved` (phase c, 2026-11-09).
   Owner, 2026-11-10: the rail's History keeps "Passed" for a confirmed merge (web says
   "Still open"; the app's is better).
9. **Settings waiting on features:** language, separate feeds, display timezone, Privacy's
   three personal-feed switches.
10. **Not compared yet:** the remaining signed-in screens (compare each from web's source).
   The right rail's Help Requests was compared and ported 2026-11-10; Settings and the profile
   page 2026-11-10 (the profile photo now opens in the PhotoViewer). Settings' Account lacks
   web's **Deactivate account** (signed out for good; content stays under the name): ADR 0088
   (owner 2026-11-10: in the directory), not built yet (a directory reset).
   web's Change password doesn't apply (passkeys and the recovery phrase).
   The seven create flows were compared with web's source 2026-11-12 (see "Create flows").

## Shell (phone)
- [x] More opens web's `MobileMoreSheet` (profile or Sign in, Settings, About), not the left
  rail. Pixel-identical at 390px signed out (`tests/e2e/shell.mjs`).
- [x] Top bar buttons: same set and order as web (home, networks, map, activity, search).
- [x] Left rail (networks), signed out: same sections (Collective, Channels, Communities,
  Discover, Feedback).
- [x] Map button: web's map panel (`features/map/MapPanel.svelte`, radius, time, filters),
  and `/map` (M6 phase 5a, ADR 0085, 2026-11-08). "Open as list" (the Region feed) is 5b.
- [x] Activity (calendar) button: web's right rail (`RightRailPanel`): Project Activity &
  Events (upcoming activities in your projects' Activity phase, with signups), Votes to cast
  (open phase moves, plans, updates and edits you haven't voted on, with Assess), History
  (ended activities, roles you hold, votes you cast, with the Show filter) and Cleared
  (dismiss ×, Restore, Restore all); the button's badge and the first desktop load opening it
  when there's something to do. Signed out it matches web at 390px (`tests/e2e/rail.mjs`).
  Help Requests (2026-11-10): web-backend's three kinds: your requests ("Your request"), ones
  you signed up for ("You signed up"), and open ones tagged with your public channels and
  communities (from the index's newest 50 cards there; web reads closed communities too), each
  with web's body preview and "N signed up · M needed"; History lists ended ones and current
  sign-ups (Show → Help), and help requests' activities no longer show as Project activity.
  Requests lists open service requests to services you run (ADR 0076).
  web-backend lists activities only from projects in `phase-5`, which leaves out personal
  services (Activity is their `phase-1`); the app lists both (owner, 2026-11-10: keep).
- [x] Rail link votes: web's "Link vote · <other project>" / "Sever link · …" rows (this
  project's half of a link request you haven't voted on, from `link_sides`; the other project's
  title from its directory listing) open the Links tab with the request expanded
  (`?tab=links&linkRequest=…`); cast link votes reach History. A plan vote's Assess opens
  that plan's own sheet in its phase. History's link outcome is this project's half's (web
  reads the whole request's).

- [x] Onboarding is web's auth surface: only the brand in the top bar, no rails, bottom nav or
  "+"; heading "Sign in or create an account" with web's intro line (also on account-only
  pages signed out).
- [x] The signed-out intro banner shows only on the feed (and channels/communities), as on web;
  it used to show on every page.

## Pages
- [x] About: web's tabbed reader with the owner's own copy (2026-11-10: "not so many
  sections, information on commons-based peer production, stigmergic signalling to replace
  price signals"): Why / How it works / Decisions / The network, describing the app's own
  rules (ADR 0033, 0071, 0074). Deliberately not web's text.
- [x] Search: web's own page (`features/search/SearchPage.svelte`, live suggestions, kind
  filter, result rows, empty-query chips). Kinds today: Projects (from `/index/projects?q=`)
  and Users (new gateway people index, `/index/people?q=`, refreshed every minute), so guests
  search too. Since 2026-10-30 (PR #79) also Channels and Communities (`/index/scopes`; your
  own private communities for members), Events and Help requests, in web-backend's order
  (scopes, then people, then items) and web's filter with icons (`tests/e2e/search.mjs`).
  Threads since ADR 0070, Posts since 2026-11-04; Activities wait (web's activity rows).
- [x] The "+" button follows web's `isFeedDiscoveryPath` (feed, platform, profiles), not search.

- [x] Profiles open for guests (name, standing, no actions), as on web; the device count is
  left out for guests (device data stays out of the guest allowlist).
- [x] Profile feed: web's toolbar (All/Public, Newest/Top/Oldest) and stack under the header,
  with the person's projects from the feed index (`/index/projects?creator=`).
- [x] Profile feed cards: web's `PersonalActivityCard` (author row with avatar inside the
  card), ADR 0055. Avatars show initials (the index has no pictures).
- [x] Followers / following counts with people sheets, the Follow / Requested / Following
  button and the Follow requests card on your own profile (M5a, ADR 0054).

## Feed
- [x] Toolbar: web's scope menu (Home / Global) first, then the audience menu (Following
  only / Following + popular) signed in (ADR 0055). Home = joined channels/communities
  merged with the projects of people you follow, on web's personal cards.
- [x] Home scope: web's Home rows are the items tagged with your joined channels and
  communities, plus the events listed on your private communities (2026-10-31: the index's
  `tag` takes several, comma-separated; `homeFeedState.svelte.ts`). The private-community
  events ignore the time window (the index's cards carry no schedule either).
- [x] Toolbar: Region in the scope menu (M6 phase 5b, 2026-11-08). ("Following + popular"'s
  discovery posts and threads: done 2026-11-04.)
- [x] Cards: comments count pill (opens the Chat tab) and the "Latest: …" update line, from
  the gateway index (`commentCount` counts up to the chat's 200-message page).
- [x] Cards: layout already matched (channel/community tags top-left, author + members + date
  bottom-right).
- [x] Cards: web's ⋮ `ReportControl` (Copy link, Report, the open report with No / Yes) on
  every card and detail header; "Under review" / "Hidden" beside it; hidden cards blur behind
  web's serious-harm banner; removed items leave the feed (2026-11-02, ADR 0071 part a).
- [x] Event cards (M5c, ADR 0057): web's `EventCard` (place line, signals, comments) linking to
  `/events/<id>` (as do the rail, notifications, search and personal cards). The card's time
  (2026-11-05): the leading approved plan's start and place, as web-backend's
  `scheduled_at` and `location_label`; placeholder places ("TBD") are left out (web's `isImplementedScheduleLabel`).
- [x] Events, part 2 (ADR 0061): Audience step, invite-only events, organizer-controlled
  governance with co-organizers; web's EventMembersPanel (Organizers/Members, Promote to
  organizer, Invite +), "Edit event"/Save and web's event phase-menu copy (2026-10-29).
- [x] Events' private-community audience (2026-10-31, PR #82, ADR 0069): web's Audience option and
  Who step (one private community + organizers); the event is listed on the community, whose
  page shows it to members, who join without an invite.
- [x] Event activities limited to the leading plan's days (2026-11-04): marked plan days,
  the wizard bounded to the chosen day, no new activities once the plan has ended.
- [x] Events' "Remove organizer" (2026-11-05, ADR 0073): web's danger button on co-organizer
  rows; a revoke-tagged `Organizers` link replayed by every reader (no DNA change).
- [x] Help requests (ADR 0067, 2026-10-29): web's create flow (six steps; tags until
  channels/communities exist), detail page (Details: time, place, roles with Sign up / Leave /
  Full, vote strip; Chat), `HelpRequestCard` in the feed with time, place and sign-ups.
- [x] Help requests' Share menu (2026-10-31, PR #80): copy link, device share, share links, and send to
  people (a direct-chat message plus web's `hr-share` notification, ADR 0067 "Sharing").
  The required channel tag came with M5d (PR #73).
- [x] Help requests on web's `PersonalHelpRequestCard` in the Home feed's personal rows and on
  profiles (2026-10-31, PR #81; `components/cards/PersonalSpaceCard.svelte` picks the card by mode).
- [x] web's project-activity cards (`ProjectActivityCard`: activity title,
  parent project, time, place, distance) come only from web-backend's Region feed
  (`feeds/region.py`): done with the Region scope (M6 phase 5b, 2026-11-08). web's feed has no
  plan cards.

## Settings (compared with web's `SettingsPage.svelte` source)
- [x] Appearance: type size, theme (light/dark/system), accent colour, font, kept on this device
  under web's storage keys (web keeps them server-side too; the app has no settings store).
- [ ] Appearance rows waiting on features: language (web: "coming soon"), separate Public and
  Personal feeds (needs the personal feed), display timezone (needs scheduled times, M5).
- [x] Regional (default location, device GPS, network location): done with M6 (ADR 0078–0085).
- [x] Profile: photo (change/remove), username, home area, bio (M1/M5a). The profile page's
  photo opens full screen in web's PhotoViewer ("View <name> profile photo", 2026-11-10).
- [x] Notifications: web's switches for the categories the app has kinds for (Comments, Votes
  needed, Phase finished), per device; off hides those cards and their bell count.
- [x] "Plan became leading" (`prj-plan-lead` / `evt-plan-lead`, off by default as on web), from
  the governance zome's `plan_lead_changes` (ADR 0072, 2026-11-03). Shares/invites and Roles
  switches exist; their remaining kinds come with their features.
- [x] Follows notifications switch (M5a).
- [x] Follow requests and Privacy's "Require approval to follow you" (M5a). Privacy's three
  "hide … from non-followers / personal feeds" switches wait for personal posts and the
  personal feed.

## Create flows (compared with web's `features/create/**` source, 2026-11-12)
- [x] Each flow's scope step has web's own layout: a project's primary channel then "Add more
  channels or communities"; an event's "Channel tags" then "Add communities" (no primary
  tag); a help request's required "Primary tag type" with always-shown additional fields
  ("Type to add another channel", "Optional."); a thread's as before. The extra fields open
  by themselves when they already hold tags, as on web.
- [x] The Overview's tag row reads web's "N channels · M communities" (projects, public
  events, help requests), not "N tags".
- [x] "Still needed:" uses web's capitalised field names (Title, Description, Location,
  Channel). The app also names what blocks steps where web only disables Continue (an
  event's plan, community or invites; a thread's or scope's fields), in the same style.
- Not parity: web's `?channel=` / `?community=` prefill (only its dead `ScopeCreateMenu`
  builds those links), and web's "a personal service cannot use the Platform channel" (the
  app has no Platform channel; feedback lives on the Feedback page).
- Not compared on screen: web's create pages need an account on the FastAPI site.

## Project page: Activity phase (M5b, ADR 0056)
- [x] Calendar card with Add activity, the creation wizard, agenda rows by day, the activity
  sheet with roles (Take/Leave, Suggest someone, Decline), Share activity.
- [x] Place search in the wizard (web's LocationPicker with quick picks): M6 phase 3. Times in the browser's zone until a Display timezone setting exists.
- [x] Past activity: web's ActivityHistorySection for self-planned activities (check-in,
  ratings, Community). The requester side waits on service requests.
- [x] Activity notifications: "… was added." (members), suggested for a role, role taken (to
  the activity's author), Settings' Roles switch, and web's "<role> suggestion declined" card
  to the suggester (the activities view lists declines per role; coordinator only).
- [x] Software governance panel (pull requests) in a software project's Activity phase
  (2026-11-08, ADR 0086). Its rail rows, history cards and `pr-approved`: phase (c),
  2026-11-09, ADR 0087.
- [x] History tab (web's `ProjectHistoryTab` / `EventHistoryTab`: Active Decisions and
  Resolutions of edits, updates, phase moves, returns and plans, each opening its vote sheet),
  on projects and events (2026-11-03, PR #90, `components/governance/HistoryTab.svelte`).
- [x] Photos and files in project chats (ADR 0056 ride-along).

## Notifications and Messages pages
- [x] Notifications page: same header, intro, "Mark all read" and card list as web's
  `NotificationsPage.svelte` (compared from source), with follow cards and their
  Accept follower / Decline actions (M5a).
- [x] Feedback page: web's `FeedbackPage` (filter, sort, compose and detail sheets, votes),
  readable by guests (M5a).
- [x] Messages: ported in M4c (ADR 0043–0051); no new differences found.

## Desktop (1280px, compared with live web signed out)
- [x] Shell, rails, top bar, intro banner, feed toolbar and cards match.
- [x] Card tag row (2026-10-29): each scope tag has its kind's icon (channel/community/
  platform, `FeedCardTop` + `ScopeGlyph`) and opens its page; "+N" opens web's `ScopeSheet`.
- [x] Channels and communities (M5d, ADR 0068, 2026-10-29, parts a–c): create pages, scope
  pages with join/members and their tagged items, create-menu Surfaces, left-rail lists, the
  scope step in create flows. The Discover sheet (PR #75), closed communities and invites, and
  the settings and vote sheets (PR #77) followed.

## Whole web pages the app doesn't have yet (route diff, 2026-10-31)
- [x] Posts and threads (2026-10-31, ADR 0070): `create/post`, `create/thread`, `/posts/<id>`,
  `/threads/<id>` (commons spaces; web's DiscussionPanel with nested replies, comment votes and
  the Oldest/Newest/Top sort; Share; ThreadCard in the public feed, PersonalPostCard on
  personal rows and profiles; Threads in Search; Post and Thread in the create menu).
- [x] @mentions in the comment composer (2026-11-04): web's MentionMenu, mentions link to
  profiles.
- [x] Links in post bodies (2026-11-04): web's `LinkedPostBody` auto-links URLs and app paths.
- [x] Home's personal feed as web-backend's (2026-11-04): your own posts, and "Following +
  popular" adds others' posts and threads (web's discovery rows). Not parity: web's
  `ScopeCreateMenu` (dead code on web, imported nowhere).
- [x] Posts in Search (2026-11-04): web's Posts filter and Post rows; the index serves posts
  to Search only (`/index/projects?posts=true`), public feeds still leave them out.
- [x] Service requests, part (a) (2026-11-06, ADR 0076, project + chat reset): a personal
  service's owner note, "Not taking requests.", Requests / Your requests (Accept, Decline with
  a reason, Cancel request, Open conversation), Request service and Request settings sheets;
  the request and its status lines live in the direct chat with the creator; `svc-request`
  cards; the right rail's Requests rows. web's `features/requests` frames are dead code.
- [x] Service requests, part (b) (2026-11-06, ADR 0077, project reset): Add availability
  (One-off / Weekly), weekly hours with Remove, the calendar and agenda of slots (Open / Taken /
  Yours / Closed; the creator's Booked / Open), requesting a slot with its times locked,
  off-schedule refusals, bookings with the Provider role, Past activity's Request history and
  Self planned history (the requester's check-in and rating), the create flow's "Service
  request mode". Not ported: web's Timezone selects (no Display timezone setting yet).
- [x] Service requests, part (c) (ADR 0089, 2026-11-10): collective services' Settings ▾ → Request settings vote,
  Request service, Plan request; any member decides (owner 2026-11-10; web's review half is
  unreachable there).
- [x] Moderation part (a), 2026-11-02 (ADR 0071): reports on projects, events, help requests,
  threads and posts (web's thresholds, Full members' votes, replayed by every reader).
- [x] Moderation part (b), 2026-11-03 (PR #88, ADR 0071): reports on comments (threads, posts) and on
  project, event and help request chat messages: the comment's ⋮, the bubble menu's Report /
  Vote to remove / Vote to keep, web's "Removed for …" and serious-harm reveal. No reset.
- [x] Moderation, later parts (2026-11-05, ADR 0075, conversation reset): reports in direct and
  group chats from the bubble menu (web's `message` target): chat members vote, the audience
  is the other members, and in a 1:1 chat the report itself removes the message (web's
  `is_tiny_private_dm`). Batched with it: declined invites are a private inbox entry.
- [x] Bot reviews and the paused badge (2026-11-05, PR #103, ADR 0074, directory reset): web's Bot
  action, "Mark … as a possible bot?", the bot review card (Real person / Bot, tally, closes,
  "Found to be a bot", Ask for a re-review), the Trust sheet (Vouchers / Bot marks tabs) from
  the standing chip, Paused standing, the `bot-review` / `bot-review-result` cards and the
  Paused badge after author names. Not ported: `bot-review-vote` to everyone sharing a space.
  The author's warm-up penalty after a removal is not parity: web runs with it off
  (`GOVERNANCE_REMOVAL_PENALTY_HOURS=0`) and the app has no warm level (ADR 0074).
- [x] A removed project's side effects (2026-11-05, UI only): it leaves its members' rail and
  they (and role holders) get web's `prj-removed` card (ADR 0071 Consequences).
- [x] The blur on hidden detail pages (2026-11-03): web's `ModerationRestrictionNotice`
  (`components/moderation/RestrictionNotice.svelte`) over the title of project, event and help
  request pages, a thread's title and body and a post's body; Reveal / Hide again.
- [x] Map panel + places (M6): Region scope, LocationPicker, Regional settings,
  project-activity cards (done 2026-11-08 with phase 5b, ADR 0078–0085).
- [x] Software project governance (pull requests / merge flow) in the Activity phase
  (2026-11-08, ADR 0086; rail rows, history cards and `pr-approved`: 2026-11-09, ADR 0087).
- [ ] **The Platform channel** (found 2026-11-12): web's `/platform` is the `platform`
  channel's scope page (`DirectoryFeedLayout`: description "The channel for the whole
  network…", Follow, its projects and events; web-backend `services/platform.py` creates the
  channel on demand). Projects tagged `#platform` take the whole network as their vote
  population (`gates.py`: any signed-in user votes; `resolve_project_vote_population`), and a
  personal service can't use the tag. The app reserves the slug (`domain/scopes.ts`), shows a
  stub at `/platform`, and links `#platform` tags there. Needs an ADR (who may create the
  channel record; the population rule is a governance change). The board (`?board=1`) is
  asset management: not parity.
- Not for parity (owner, 2026-10-31): asset management (`platform/assets`, attached assets,
  the platform board, asset-use requests) comes later.
- web's `personal`, `login`, `signup`, `welcome`, `scopes/invites/redeem` are covered by the
  app's Home scope, onboarding and `/invites`; `profile/[username]` by `profile/[agent]`.

## Not compared yet
Signed-in screens other than Settings and Notifications (compare from web's source; don't create
accounts on the FastAPI site). Everything left above waits on features: the personal feed, channels and
communities (M5), places and moderation (M6).
