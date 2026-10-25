# 0052. Guests open project pages: the gateway's archivist answers a short list of reads

Status: accepted (2026-10-24). Builds on [0031](./0031-archivist-and-node-index.md) and
[0036](./0036-every-full-node-keeps-the-network.md).

## Context
web shows a project page to anyone: a signed-out visitor reads Details, Participation, Chat and
Links, and every action (Join, Support, Send) calls `requireViewer`, which shows
`AuthActionNotice` ("Sign in to join this project." with a Sign in link). The app's guests
could browse the feed (served by `/index/projects`) but a project page needs a cell, and a
guest has no agent, no cell and no websocket.

## Decision
- **`POST /public/read` on the gateway** (`routes/public.rs`, `node/public.rs`): `{ seed?, zome,
  fn, payload }` with a msgpack payload; the archivist (which has joined every listed project)
  makes the call and the answer is msgpack, exactly what a zome call returns. Only an
  allowlist is served: the directory's `get_profile`, `get_standing`, `get_listing`,
  `find_listing`; in a project `get_project`, `list_proposals`, `list_plans`, `list_values`,
  `link_sides`, `list_links`, `list_messages`. Everything else is 403, a project the archivist
  hasn't joined is 404 (the page retries, as it does for a fresh project). Chats (the
  conversation DNA) and device data are never served. 20,000 reads per IP per day.
- **The UI reads through the same services.** `lib/holo/guest.ts`: signed out, `callZome` on
  the directory and `callCell` on a project go to `/public/read`; `projectCell` returns a
  stand-in cell id that remembers the seed. Services and pages don't know the difference;
  the archivist isn't a member, so the page shows a non-member's view.
- **web's notice is ported** (`features/shell/AuthActionNotice.svelte`, `stores/authNotice`
  `requireViewer`, `styles/notice.css`). Join and Support on the project page, Support on feed
  cards (which used to jump to /onboarding) and the chat composer use it; the composer shows
  web's "Sign in" button in place of Send.

## Consequences
- Guest reads cost the seed node's CPU; a page load is ~10 calls and the open chat polls every
  5 s. The per-IP cap stops one address from hammering it. A gateway that wants none can drop
  the route.
- The answers are whatever the archivist's node has gossiped, so a guest may see a project a
  few seconds behind a member.
- @names in chat bubbles stay plain text for guests (the mention pool needs your own chats).
