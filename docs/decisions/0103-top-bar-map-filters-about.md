# 0103. The top bar, the map's filters, About, and the way back from sign-up

Status: **accepted** (2026-11-14, owner, from mockups; the map's filters as "style A, with
the search as a button on the main bar").

## Context
After ADR 0098 the top bar was a logo and four loose icons: nothing said which panel was open
and search hid behind a tap. The map's filters were seven icon-only buttons in one bar. About
opened with a very large headline over four tiny boxed tabs. On a phone, the only way from
sign-up back to the feed was the logo, which does not look like a button.

## Decision
No feature is added or removed, except the two small sign-up helpers below.

- **Top bar.** At every width: the logo, a search field you can type in at once (a soft
  pill; the phone's "open search" button is gone), and Networks, Map and Activity together
  in one soft capsule whose open button lifts to the panel colour. On desktop the capsule
  sits beside the logo, apart from Home, Notifications and Messages (owner, 2026-11-14:
  mixed together they read as one group); About, the profile and Settings stay at the right.
- **Map filters (phones and fullscreen).** One capsule over the top of the map: a search
  button, the distance, "Upcoming" as an on/off chip with its word, and "Filters". Search
  opens a second bar under the capsule (the place field and "Use my location") only when
  asked, so the map stays visible. Filters opens a sheet with the time window (and custom
  dates) and the type, as the feed's sheet does. "Open as list" moves to the results'
  header ("3 near here · List"); in fullscreen, which has no results, it stays in the
  capsule. The desktop bar under the map is unchanged.
- **About.** The headline is smaller and balanced, the four sections are one segmented
  control that stays under the top bar while you read, and each section opens with its
  titled parts as summary cards (the title and the first sentence of its text). The words
  do not change.
- **Sign-up and log-in.** The top bar shows a "Back to feed" pill where the bare logo was,
  and "Keep browsing without an account" sits under the form. The recovery phrase step has
  "Copy all 24 words" (owner, 2026-11-14: to paste them into a document).

## Consequences
- UI only: no zome or network change. Styles: `styles/ux/topbar.css`, `mapfilters.css`,
  `about.css`; the capsule is `features/map/MapFilterCapsule.svelte`.
- The rest of the sign-up screens' new look comes with the chats / profile / onboarding
  group (approved 2026-11-13).
