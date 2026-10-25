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
- [ ] Activity (calendar) button: web opens the right rail's activity; the app shows "Your
  activity will appear here." Comes with M5 activities.

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
- [ ] Search: web's own page (`features/search/SearchPage.svelte` + `SearchSuggestionsList`,
  `liveSearch.ts`, `formatSearchMeta.ts`, `searchKinds.ts`: heading, search box with live
  suggestions, kind filter, result rows with a kind chip). The app shows the feed filtered by
  the query. Plan: kinds the app has today are Projects and Users (keep web's filter list but
  show only those two until the others land). Projects: `/index/projects?q=`. Users: add a
  people index to the gateway (the archivist already reads profiles each round; serve
  `/index/people?q=` with name + key + standing), so guests can search too without
  `list_people` in the guest allowlist. `suggestedQueries` (web's empty-query chips): web's
  backend picks popular tags; use the index's most common tags.

## Feed
- [ ] Toolbar: web's scope menu (globe: Home / Global / Region) comes first. The app is Global
  only: Home needs follows and joined communities, Region needs place coordinates (M6 map).
  Port the menu when either lands.
- [x] Cards: comments count pill (opens the Chat tab) and the "Latest: …" update line, from
  the gateway index (`commentCount` counts up to the chat's 200-message page).
- [x] Cards: layout already matched (channel/community tags top-left, author + members + date
  bottom-right).
- [ ] Cards: web's ⋮ `ReportControl` (copy link, report) top-right. Comes with M6 moderation;
  "copy link" alone could come sooner.
- [ ] Event, activity and plan cards (web's feed mixes them); the app lists projects only (M5).

## Not compared yet
Profile pages (own and others'), settings, notifications, messages at 390px
against web signed in (needs a web account: don't create one on the FastAPI site; compare
markup in web's source instead), desktop widths.
