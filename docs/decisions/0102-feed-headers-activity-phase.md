# 0102. Feed headers and the activity phase: tabs, a channel-first row header, a week strip

Status: **accepted** (2026-11-14, owner, from mockups: "lets use all your suggestions").

## Context
The UX pass (ADR 0096 to 0100) had not reached three places. The feed opened with a card of
five icon-only menus (scope, audience, filter, sort, window): each had to be tapped to learn
what it was. A feed row led with one grey line (type, tag, clock, then the phase and ⋮).
The Activity phase gave most of its first screen to a boxed month grid, listed activities as
striped rows, and kept past activity in a fold at the bottom.

## Decision
No feature is added or removed; every control is still reachable.

- **Top of the feed.** Home, Global and Region are word tabs (`FeedHeader`). One button
  opens a sheet, "Show in Home", with the other four menus under their names (From, Type,
  Order, When) as segmented choices (`SegmentedChoice`); a dot on the button says a type or
  time filter is on. Region keeps its radius, place, location and online controls in a row
  under the tabs: a place search needs room and shows where you are looking.
- **Feed row header.** A row leads with its type's glyph in a soft square of the type's
  colour and its first channel in bold; the type, phase and clock sit underneath, the private
  mark and ⋮ at the right. Further tags stay behind "+N". Detail pages keep their one-line
  top (`FeedCardTop` switches on `createdAt`, which only feed rows pass).
- **Activity phase.** "Upcoming" and "Past activity" are tabs (shown once there is past
  activity). The calendar is one week of soft columns; the month's name opens the whole
  month and folds it back onto the picked day's week, and the arrows move by a week or a
  month to match. An event's plan days stay the only days to pick. Activities are soft cards
  grouped by day with the first three people in, the fill chip, and "Join" while a role has
  room (the card opens the activity's sheet, where roles are taken, as before).

## Consequences
- UI only: no zome or network change.
- Browser tests reach the feed's menus through `feedScope` and `feedChoice` (`tests/e2e/lib.mjs`).
- A service's calendar (`ServicePhase`) keeps the month grid; it goes with personal services
  (ADR 0059).
