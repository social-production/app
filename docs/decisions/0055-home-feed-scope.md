# 0055. The feed's Home scope: the projects of people you follow, merged as on web

Status: **accepted** (2026-10-26). Follows ADR 0054 (follows) and ADR 0031 (the node's index).

## Context
web's public feed (`features/public-feed/PublicFeed.svelte`) starts its toolbar with a scope
menu (Home / Global / Region) and, signed in, an audience menu (Following only / Following +
popular). With web's `combineFeeds` setting (on by default) the page merges two lists with
`utils/homeFeed.ts` `mergeHomeFeed`: the public rows of the chosen scope (Home = items tagged
to the channels and communities you joined, web-backend `get_home_feed`) and the personal
feed (`get_personal_feed`: what the people you follow made; "popular" adds discovery posts
and threads). Personal rows use web's `PersonalFeedCard`s, which also fill profile feeds. The
app had a Global feed of projects only and none of this.

## Decision
- **Port the menus and the merge.** Scope menu Home / Global; Region joins it with places (M6),
  as other menus list only what exists. Audience menu shown signed in, as on web.
- **`combineFeeds` is always on** (web's default): the app has no setting for it yet
  (PARITY's "separate Public and Personal feeds" row).
- **Home's public rows are empty until channels and communities exist (M5d)**, so Home shows the
  personal feed: the projects of people you follow (accepted follows, `followCircles`), from the
  node's index: `GET /index/projects?creator=` now takes a comma-separated list of keys (up to
  500). In Global the same personal rows are dropped when the public page already has them, as
  `mergeHomeFeed` does on web.
- **"Following + popular" changes nothing yet**: web's extra items are posts and threads, which
  the app doesn't have. The menu is ported anyway, so the page matches web when they land.
- **Cards:** web's `PersonalActivityCard` + `PersonalFeedIdentity` for personal rows and on
  profile feeds (PARITY). Avatars show initials: the index card carries no picture.
- **Kept per device:** scope and audience in browser storage (`sp_feed_scope`,
  `sp_home_audience`); web keeps them in its server-side settings. Signed out, the feed shows
  Global and keeps the choice for when you sign in (web snaps the scope back to Global).

## Consequences
- No zome change; gateway + UI only. Nothing resets.
- A long follow list makes a long query string (~53 characters per key; 500 keys ≈ 27 KB),
  fine for today's network; a POST or a node-side follow index replaces it if needed.
- The personal feed is as fresh as the node's index (the archivist re-reads every minute).
