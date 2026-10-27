# UX pass (after parity, before launch)

Owner notes, 2026-10-27. Parity first: until then, port from web as it is. Then this pass,
then launch (no real users before; the FastAPI site is a prototype and gets wiped).

## Cards and feed
- Remove the left type stripe on cards (generic). Show the type as icon + label on the card's
  top line (web's `SurfaceTypeLabel`, as the rail does).
- Twitter-style feed cards, larger: bigger text, a spaced action row (comments, support /
  oppose, share), full-width rows with thin dividers instead of boxed cards.
- **Collective, not owned:** public projects, events and threads show no prominent avatar;
  they should feel like collective activities, not things a user controls. Avatars only on
  personal feed cards and on profile pages (a profile page can be more personal). Avatars need
  profile photos, with a letter fallback.
- Image posts like Reddit/Twitter (not only in chats). Video later.

## Controls
- Floating controls, as in YouTube's nav and Samsung Gallery's header (owner screenshots,
  2026-10-27): the phone's bottom nav becomes a translucent, blurred capsule floating above
  the content with side margins and a soft shadow, not an edge-to-edge bar.
- On project and event pages the dock becomes floating pill groups (a round back button on the
  left, one pill of actions: Join / members / Edit / Vote / Phase).

## Polish
- Spacing, type scale, press states and transitions: the shine other social apps have.
- Ask the owner: is the green gears image (2026-10-27) the app icon?

## Creation
- Types evolve through plans; personal services go (ADR 0059).

## Code health to do with it
- "Kind profile": one descriptor per type instead of `mode === …` checks (19 UI files, 15 Rust
  places as of 2026-10-27).
- Split `detailState.svelte.ts`, `AppShell.svelte`, `ChatPanel.svelte` (280–294 lines) by
  responsibility.
- One helper for "resolved path + query" links, replacing ~31 `eslint-disable` lines.
