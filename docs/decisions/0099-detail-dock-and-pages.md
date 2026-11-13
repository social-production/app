# 0099. Project, event and request pages: the dock replaces the nav; votes fold into a chip (UX pass)

Status: **accepted** (2026-11-13, owner approved the round 3 mockup, "dock B"). Plan:
[`docs/plans/UX.md`](../plans/UX.md). Builds on ADR 0096 (the capsule and its tokens).

## Context
Inside a project, event or request a phone showed three stacked bars: Support / Oppose, the
action dock, then the nav. The owner: the detail controls should look like the nav capsule;
these pages are "very important"; no buttons in the page content of projects and events;
icons without words; no menu button.

## Decision
- **The dock takes the nav's place** on these pages (`DetailActionDock.svelte`,
  `styles/ux/dock.css`): a round back button and one floating capsule of actions. The nav
  capsule hides while a dock is in the shell. Back is the browser's back, or home.
- **One row of actions, icons without words:** members (an icon with the count once joined),
  Share, Edit, Update. Only the one call to action keeps its word (Join, Add value, Add plan,
  Add activity, Phase). There is no ⋯ menu button. On phones the capsule never wraps.
- **Voting floats above the capsule**, right-aligned on phones:
  - Support / Oppose (with their words and the percentage) until the person has a stance;
    then they fold into a **chip** in the capsule (an arrow and the support percentage, red
    when opposed). Tapping the chip opens them again to change or withdraw.
  - An open vote (an edit, an update, a phase move, request or software settings) is its own
    **Vote** pill there. The mockup had it inside the capsule; with a stance chip, members
    and three icons that did not fit 390px in one row.
- **Tabs** are plain words with a green underline on phones, starting from the left.
- **Take part:** the five-box phase strip is a thin progress line (each segment still a
  button that shows that phase) with "Phase 2 of 5 · Production Plan" under it; plan cards
  are soft rounded fills with approval as a bar (`styles/ux/takepart.css`).
- **Details:** the top line leads with the type (icon + label) before the tags; the title is
  24px and starts at the left edge; when and where are icon lines (`FactLine.svelte`).
- **Requests:** roles are rows in the page with a Sign up pill; the dock holds the votes and
  Share (`HelpRolesDock.svelte`).
- Overrides only, each rule starting with `.shell`; no behaviour changes.

## Consequences
- web's row-packing script in `PhaseWorkToolbar` is gone (the capsule lays actions out).
- The project title was centred by accident (a ported `PersonalFeedIdentity` rule, as ADR
  0067 found on requests); it is now left-aligned on every screen size.
- With no stance yet *and* an open vote, a phone shows three floating rows. Rare (a member
  who has not signalled), and each is one tap.
- The phase line no longer says "Current phase"; an earlier or later phase reads
  "· Complete" or "· Upcoming".
