# 0095. The phone's bottom nav is a floating capsule (UX pass)

Status: **accepted** (2026-11-12, owner chose option B from the mockups). Plan:
[`docs/plans/UX.md`](../plans/UX.md), "Controls".

## Context
The phone's bottom nav was web's edge-to-edge bar: four icon slots on a solid strip with a top
border. The owner wants floating controls, as in YouTube's nav and Samsung Gallery's header
(screenshots, 2026-10-27), with Twitter's phone app as the general styling reference, and
picked "B" of two mockups (icons only, or the current tab named).

## Decision
- `MobileBottomNav` is a capsule: 60px high, 16px from each side, 12px above the screen edge
  (plus the safe-area insets), fully rounded, with a translucent blurred fill, a hairline
  border and a soft shadow. The page scrolls under it.
- The current tab is a tinted pill that shows its name beside the icon ("Home",
  "Notifications", "Messages", "More"); the other tabs are bare icons that share the rest of
  the width. Tabs press in slightly. Unread badges sit on the icon's corner.
- The tabs, their order, the More button and the hide-in-chat behaviour are unchanged.
- `--shell-bottom-nav-base` grows from 56px to 80px (capsule + margin + 8px of air). Everything
  that docks above the nav (create button, detail dock, chat composer, rails, toasts) reads
  `--shell-bottom-nav-offset` from it, so it clears the capsule without its own change.
- Shared tokens for floating controls live in `styles/ux/tokens.css` (`--float-background`,
  `--float-border`, `--float-shadow`, `--float-blur`), for the next floating pieces (the
  project and event dock's pill groups, which the parity session owns).

## Consequences
- This departs from web's shell on purpose: the UX pass follows parity (owner, 2026-10-27),
  and web is not kept in step.
- Docked bars now sit 80px above the screen edge on phones, with the page visible around the
  capsule below them. That is right for the feed; the detail dock looks tall until it becomes
  floating pills too.
- `backdrop-filter` needs a browser from the last few years; without it the capsule is a
  plain translucent panel, still readable.
