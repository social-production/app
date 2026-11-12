# 0096. Feed cards are Twitter-style rows: no type stripe, no author on collective items (UX pass)

Status: **accepted** (2026-11-12, owner approved the mockups and chose to drop the author
name). Plan: [`docs/plans/UX.md`](../plans/UX.md), "Cards and feed".

## Context
Feed cards were web's: a 6px coloured stripe for the type, 13px summaries, a boxed vote strip
and comment pill in the corner, and the creator's name in the footer. The owner wants
Twitter's phone rows, the type shown as icon + label, and public projects, events and
threads to read as collective activities, not things a person controls.

## Decision
- **No type stripe** on any `FeedSurface` row (feed, map list, links lists). The owner keeps
  a stripe only on History cards, where it will mean approved or not (a later piece).
- **Top line:** the type as icon + label (`SurfaceTypeLabel`, tinted with the type's colour),
  the tags, then the clock; the private mark, the stage (a small pill) and ⋮ on the right.
- **Larger type:** titles 18px, summaries 15.5px in the main text colour.
- **Action row** (`FeedActions.svelte`): comments, the votes, members (projects and events)
  and share, as bare icon buttons spread across the row, each at least 40px. Share opens
  the device's share sheet, or copies the link where there is none.
- **Votes** keep their rules and components: `SignalStrip` and `VoteStrip` gain a `bare`
  variant (drawn arrows, no box) used by feed rows; detail pages keep the boxed strip.
- **Collective rows name no author:** project, event, thread and help-request rows in the
  public feed show neither avatar nor creator. The creator stays on the item's own page.
- **Personal rows keep the person:** a personal post is a Twitter row (avatar column, name and
  clock, body, actions); personal activity and help-request cards keep their identity row.
- The overrides live in `styles/ux/feed.css`, loaded after the ported `cards.css`
  (`port-styles.py` rewrites the ported files, so they are never hand-edited).

## Consequences
- The feed departs from web's on purpose (UX pass, after parity).
- `FeedSurface`'s `accent` prop no longer draws anything. It stays until the parity session's
  pages stop passing it, then goes (UX.md, code health).
- Browser tests that reached a profile through a feed card's author link now go through the
  item's page (`guests.mjs`, `help-requests.mjs`).
- Image posts are not part of this: a post's charter has no attachment field
  (`project_integrity::Charter`), so they most likely need an integrity change (a network
  reset) and get their own ADR first.
