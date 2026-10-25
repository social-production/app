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

## Feed
- [ ] Toolbar: web has a scope (globe) button first; the app starts at filter.
- [ ] Cards: web shows the author top-left (`ContentMetaRow`), a ⋮ menu (report) top-right, and
  a comments-count button beside the vote strip; the bottom right shows place/community,
  members and date. The app shows tags top-left and the creator bottom-right, no ⋮, no
  comments count. Check web's `ProjectCard.svelte` + `FeedCardTop.svelte` before porting.
- [ ] Event, activity and plan cards (web's feed mixes them); the app lists projects only (M5).

## Not compared yet
Search, profile pages (own and others'), settings, onboarding, notifications, messages at 390px
against web signed in (needs a web account: don't create one on the FastAPI site; compare
markup in web's source instead), desktop widths.
