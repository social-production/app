# 0100. Post and thread pages: the item as an opened feed row, flat comments on posts, a floating comment bar (UX pass)

Status: **accepted** (2026-11-13; the owner approved flat comments from the round 2 mockup,
asked for the post itself to take the feed row's style, asked for floating comment bars,
and chose the soft pills from three mockups).
Plan: [`docs/plans/UX.md`](../plans/UX.md). Builds on ADR 0096 (the capsule and its tokens),
0097 (feed rows) and 0099 (the dock in the nav's place).

## Context
After the feed rows (ADR 0097) a post or thread page still showed the ported look: a type
stripe down the left edge, a boxed vote row in the corner, small grey text, and nested boxed
comments on posts. The comment field was a full-width bar glued above the nav. Notification
cards kept their type stripe too, because the feed's overrides only loaded with feed cards.

## Decision
- **The item is its feed row, opened up, with soft pills** (owner chose mockup "A · Soft
  pills", Reddit's feel, 2026-11-13: https://claude.ai/artifact/UZFHQJuuozrxSRXd9xZ2qM,
  private to the owner; `PostHeader.svelte`, `ThreadOverview.svelte`, `styles/ux/item.css`):
  - a post leads with the author (avatar, name, when it was written under the name, ⋮),
    then the text at 17px;
  - a thread leads with its type and tags as soft chips, then a 22px title and the text at
    16px, with who started it (small avatar, name, age) underneath (owner: "type and tag at
    top, username bottom");
  - the actions are rounded filled pills with no lines around them
    (`DiscussionActions.svelte`): the votes together in one pill (tinted once you vote), the
    comment count, and Share with its word;
  - a **Comments** heading follows, with Sort as a word and a chevron at its right
    (`DiscussionTools` `only`);
  - comment actions are the same pills, smaller: the votes and "Reply" (on threads' nested
    comments too, which now use the bare vote strip).
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
- Threads and posts differ in their comments on purpose (nested against flat rows); both
  use the same pills.
