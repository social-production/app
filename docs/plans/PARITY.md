# Parity with web (the FastAPI site)

What the P2P app still lacks or does differently from web, found by comparing the two at 390px
(live web: https://web-production-885ec.up.railway.app, read-only; app: a local seed node or the
live UI). Port each item from web's own components; never invent (STATUS "Owner decisions").
Tick items off as they ship, with the PR number. Add findings as you compare more screens.

## Shell (phone)
- [x] More opens web's `MobileMoreSheet` (profile or Sign in, Settings, About), not the left
  rail. Pixel-identical at 390px signed out (`tests/e2e/shell.mjs`).
- [x] Top bar buttons: same set and order as web (home, networks, map, activity, search).
- [x] Left rail (networks), signed out: same sections (Collective, Channels, Communities,
  Discover, Feedback).
- [ ] Map button: web opens the map panel (`features/map/MapPanel.svelte`, radius, time,
  filters); the app's does nothing. Comes with M6 map/places.
- [x] Activity (calendar) button: web's right rail (`RightRailPanel`): Project Activity &
  Events (upcoming activities in your projects' Activity phase, with signups), Votes to cast
  (open phase moves, plans, updates and edits you haven't voted on, with Assess), History
  (ended activities, roles you hold, votes you cast, with the Show filter) and Cleared
  (dismiss ×, Restore, Restore all); the button's badge and the first desktop load opening it
  when there's something to do. Signed out it matches web at 390px (`tests/e2e/rail.mjs`).
  Help Requests and Requests stay empty until help requests and service requests exist (M5c).
  **Owner review:** web-backend lists activities only from projects in `phase-5`, which leaves
  out personal services (Activity is their `phase-1`); the app lists both.
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
- [x] About: web's About page with its tabs, ported with web's copy unchanged (pixel-identical
  at 390px). **Owner review:** the copy describes web (Roadmap "Now: Web prototype, FastAPI
  backend"; quorum from weekly actives, while the app counts full members, ADR 0033). Say if
  the P2P app should say something different.
- [x] Search: web's own page (`features/search/SearchPage.svelte`, live suggestions, kind
  filter, result rows, empty-query chips). Kinds today: Projects (from `/index/projects?q=`)
  and Users (new gateway people index, `/index/people?q=`, refreshed every minute), so guests
  search too. The filter lists only those two until the others land; web's filter options
  show icons, the app's `IconMenuButton` has none yet.
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
  only / Following + popular) signed in (ADR 0055). Home = joined channels/communities (none
  until M5d) merged with the projects of people you follow, on web's personal cards.
- [ ] Toolbar: Region in the scope menu (needs place coordinates, M6); "Following + popular"
  adds web's discovery posts and threads once they exist.
- [x] Cards: comments count pill (opens the Chat tab) and the "Latest: …" update line, from
  the gateway index (`commentCount` counts up to the chat's 200-message page).
- [x] Cards: layout already matched (channel/community tags top-left, author + members + date
  bottom-right).
- [ ] Cards: web's ⋮ `ReportControl` (copy link, report) top-right. Comes with M6 moderation;
  "copy link" alone could come sooner.
- [x] Event cards (M5c, ADR 0057): web's `EventCard` (place line, signals, comments) linking to
  `/events/<id>` (as do the rail, notifications, search and personal cards). Still to come:
  the time on the card (the index needs the leading plan's schedule).
- [x] Events, part 2 (ADR 0061): Audience step, invite-only events, organizer-controlled
  governance with co-organizers; web's EventMembersPanel (Organizers/Members, Promote to
  organizer, Invite +), "Edit event"/Save and web's event phase-menu copy (2026-10-29).
- [ ] Events, later parts: "Remove organizer" (revocable grants: an integrity change),
  private-community audience (M5d), activities limited to the plan's days, History tab.
- [x] Help requests (ADR 0067, 2026-10-29): web's create flow (six steps; tags until
  channels/communities exist), detail page (Details: time, place, roles with Sign up / Leave /
  Full, vote strip; Chat), `HelpRequestCard` in the feed with time, place and sign-ups.
- [ ] Help requests, later: web's Share menu on the request (copy link + send to people), the
  required channel/community tag (M5d), `PersonalHelpRequestCard` in the personal feed.
- [ ] web's project-activity cards (`ProjectActivityCard`: activity title,
  parent project, time, place, distance) come only from web-backend's Region feed
  (`feeds/region.py`), so they wait on Region (places, M6). web's feed has no plan cards.

## Settings (compared with web's `SettingsPage.svelte` source)
- [x] Appearance: type size, theme (light/dark/system), accent colour, font, kept on this device
  under web's storage keys (web keeps them server-side too; the app has no settings store).
- [ ] Appearance rows waiting on features: language (web: "coming soon"), separate Public and
  Personal feeds (needs the personal feed), display timezone (needs scheduled times, M5).
- [ ] Regional (default location, device GPS, network location): needs places (M6).
- [x] Notifications: web's switches for the categories the app has kinds for (Comments, Votes
  needed, Phase finished), per device; off hides those cards and their bell count.
- [ ] Notifications still to come: web's "Plan became leading" (off by default; the app could now
  derive `prj-plan-lead`, ADR 0046 left it out for lack of settings), shares/invites, roles.
- [x] Follows notifications switch (M5a).
- [x] Follow requests and Privacy's "Require approval to follow you" (M5a). Privacy's three
  "hide … from non-followers / personal feeds" switches wait for personal posts and the
  personal feed.

## Project page: Activity phase (M5b, ADR 0056)
- [x] Calendar card with Add activity, the creation wizard, agenda rows by day, the activity
  sheet with roles (Take/Leave, Suggest someone, Decline), Share activity.
- [ ] Place search in the wizard (web's LocationPicker with quick picks): a plain label until
  places land (M6). Times in the browser's zone until a Display timezone setting exists.
- [x] Past activity: web's ActivityHistorySection for self-planned activities (check-in,
  ratings, Community). The requester side waits on service requests.
- [x] Activity notifications: "… was added." (members), suggested for a role, role taken (to
  the activity's author), Settings' Roles switch, and web's "<role> suggestion declined" card
  to the suggester (the activities view lists declines per role; coordinator only).
- [ ] Software governance panel (pull requests) in a software project's Activity phase.
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
- [ ] Card tag row: web prefixes each scope tag with its kind's icon (channel/community/
  platform, `FeedCardTop` + `ScopeGlyph`) and links it to the scope; the app's chips still
  show `#slug`/`@slug` text (M5d part c follow-up).
- [x] Channels and communities (M5d, ADR 0068, 2026-10-29, parts a–c): create pages, scope
  pages with join/members and their tagged items, create-menu Surfaces, left-rail lists, the
  scope step in create flows. Still to come: required channel tag, Discover sheet, closed
  communities and invites, settings and vote sheets.

## Not compared yet
Signed-in screens other than Settings and Notifications (compare from web's source; don't create
accounts on the FastAPI site). Everything left above waits on features: the personal feed, channels and
communities (M5), places and moderation (M6).
