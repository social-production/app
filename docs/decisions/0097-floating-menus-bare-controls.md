# 0097. Bare icon buttons, floating create and More menus, outcome stripes in the rail's History (UX pass)

Status: **accepted** (2026-11-12, owner approved the round 2 mockups). Plan:
[`docs/plans/UX.md`](../plans/UX.md). Builds on ADR 0095 (the capsule and its tokens).

## Context
The owner: Twitter's phone app is the reference; icon buttons should have no labels and no
boxes (the top bar's map and rail buttons were bordered squares); the create menu and the
phone's More menu "could be so much better" (both covered the whole screen on phones); the
right rail's cards should lose the type stripe, with a stripe only on History cards, meaning
approved or not; and the theme should follow the device unless the person chooses.

## Decision
- **Icon buttons are bare:** the top bar's networks, map, activity, search and settings
  buttons lose their borders; the active one is a tinted circle and they press in slightly.
  The top bar is translucent and blurs the page under it.
- **Map:** the filter row is a floating capsule in the nav's style, its buttons and the radius
  picker bare; the full-screen button and the zoom group are round floating controls.
- **Left rail:** Discover and Feedback are pills.
- **Create menu:** a floating blurred panel anchored above the plus button on every screen
  size, one line per item (icon + name; no descriptions, no section titles, a hairline
  before Community and Channel). The plus stays in place and shows a close mark.
- **More menu** (`MobileMoreSheet.svelte`): a floating panel above the nav capsule with the
  person's avatar and name ("View profile"), Settings, About, and a System / Light / Dark
  switch. Signed out: Sign in / Sign up, About and the switch. The create button hides while
  it is open. Same dialog name and links as before.
- **Theme default is System** (was Light): a new visitor gets the device's setting
  (`appearance.svelte.ts`, and `app.html` before first paint).
- **Right rail:** cards have no type stripe (the top line's icon + label says the type);
  their buttons are pills. **History rows carry a 4px stripe for the outcome:** green passed,
  red not passed, grey for everything else.
- All of it is overrides in `styles/ux/{controls,menus,rail}.css`, imported through
  `styles/ux/index.css`. Each rule starts with `.shell` so it beats the ported rule whatever
  order the stylesheets load in (ported files load with their components, some lazily).

## Consequences
- The ported `shell/more-sheet.css` is no longer imported (its PORTS row stays, so
  `port-styles.py` keeps regenerating it); remove the row when web is retired.
- The theme switch is the first control in the shell that web does not have.
- "Not passed" in History is now red (`--danger`), not warm orange.
