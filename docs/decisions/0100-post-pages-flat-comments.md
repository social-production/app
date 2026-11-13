# 0100. Post and thread pages: the item as an opened feed row, flat comments on posts, a floating comment bar (UX pass)

Status: **accepted** (2026-11-13; the owner approved flat comments from the round 2 mockup,
asked for the post itself to take the feed row's style, and asked for floating comment bars).
Plan: [`docs/plans/UX.md`](../plans/UX.md). Builds on ADR 0096 (the capsule and its tokens),
0097 (feed rows) and 0099 (the dock in the nav's place).

## Context
After the feed rows (ADR 0097) a post or thread page still showed the ported look: a type
stripe down the left edge, a boxed vote row in the corner, small grey text, and nested boxed
comments on posts. The comment field was a full-width bar glued above the nav. Notification
cards kept their type stripe too, because the feed's overrides only loaded with feed cards.

## Decision
- **The item is its feed row, opened up** (`PostHeader.svelte`, `ThreadOverview.svelte`,
  `styles/ux/item.css`):
  - a post leads with the author (avatar, name, ⋮), the text at 17px, then when it was
    written ("7:01 pm · 13 Nov 2026", `postedAt`);
  - a thread leads with the type and tags line, a 22px title, the text at 16px, then who
    started it and when;
  - one action row between two hairlines (`DiscussionActions.svelte`): the comment count,
    the bare votes, Share and Sort, all bare icons as in the feed. On wide screens they sit
    together at the left instead of spreading.
- **No type stripe** on post and thread pages or on notification cards (`styles/ux/index.css`
  imports `feed.css`; the page's stripe was a painted background, removed in `item.css`).
- **Comments on posts are flat** (`DiscussionPanel` / `DiscussionComment` `flat`,
  `styles/ux/comments.css`): an avatar column, name and time, the comment, bare votes; a
  reply sits at the same level under its parent, joined by a line, and says "Replying to
  …". **Threads keep the ported nested comments.**
- **The comment bar floats in the nav's place on phones**, as the dock does on detail pages:
  a round back button (`BackButton.svelte`, shared with the dock) and the composer as one
  blurred capsule that grows with the text; the send button is round. A reply's bar floats
  the same way, without the back button (its ✕ is on the comment). The nav capsule hides on
  these pages. Desktop keeps the composer in the page above the comments.
- The right rail's type label takes its type's tint, as in the feed (a ported rule greyed
  its inner span; `styles/ux/rail.css`).
- Overrides only, each rule starting with `.shell`; no behaviour changes.

## Consequences
- The comment count on these pages is an icon and a number; its words ("2 comments") are
  the accessible label, so tests read it by label, not by text.
- A phone has no nav on a post or thread page; the back button or the top bar leaves it.
- Threads and posts now differ in their comments on purpose (nested boxes against flat
  rows); a thread's comments have not taken the bare vote style.
