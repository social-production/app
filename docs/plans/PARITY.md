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
- [ ] About: web's long About page with tabs (Purpose, Strategy, Roadmap, Costs, Gov, Mod,
  Join) (`web/src/lib/features/about/*`, `routes/about`); the app has a one-paragraph stub.
  Static content: port it as is, then adjust only facts that differ for the P2P app.
- [ ] Search: web's own page (`features/search/SearchPage.svelte`: heading, search box with
  live suggestions, kind filter All/Projects/…/Users, result list). The app shows the feed
  filtered by the query. Projects come from `/index/projects?q=`; users from the directory.

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
