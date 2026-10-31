# 0067. Help requests are commons spaces with one activity

Status: **accepted** (2026-10-29). Follows [0056](./0056-activities-and-roles.md),
[0057](./0057-events-as-project-clones.md) and [0065](./0065-public-commons-network.md). M5c's second half.

## Context
web's help requests (`features/help-requests`, `features/create/help-request`,
`HelpRequestCard`; web-backend `app/services/content/help_requests.py`) are one-off asks:
title, body, place, a start time (and optional end), roles with slots (0 = any number), tags,
up/down votes, a chat, and people committing to one role each (`409` if already in one, or if
the role is full). No phases, no proposals, no plans.

The app already has everything except the kind: commons spaces (charter + members + chat +
signals, ADR 0065) and activities with roles and `Assignments` links (ADR 0056).

## Decision
- A help request is a **commons space whose charter mode is `help-request`** (new crate
  `sp_help`, so `sp_core` and the other DNAs keep their hashes). It stays in `phase-1`: the project coordinator's view gives it no visible phases, no next
  phase and no returns, and the governance coordinator refuses proposals and plans in it.
  `sp_governance` stays untouched: the directory DNA depends on it, so changing it would reset
  accounts too. (Integrity still treats the mode like `productive` for proposals, as it does
  events' field limits; a peer bypassing the coordinator could file one, which nothing reads.)
- Its schedule, place and roles are **one `Activity`** the creator writes when founding it
  (title = the request's title, note = its body). Integrity: in a help request only the space's
  creator adds activities. Roles map web's slots: `slots > 0` → `required = maximum = slots`;
  `slots = 0` (any number) → `required = 1`, no maximum. `ActivityRole` gains an optional
  `description` (web's role description), `#[serde(default)]`.
- **Committing** joins the space (its `Members` link) and takes the role (`Assignments`), as
  activities do; the coordinator refuses a second role in the same request (web's 409), and
  `sp_activity::fill` already decides who holds the last place.
- **Votes** (web's up/down `VoteStrip`) are the space's Support/Oppose signal; the count is
  support minus opposition. **Comments** are the space's chat (discussion zome).
- **Tags:** web-backend requires a channel or community tag. Until channels and communities
  land (M5d) the app's existing tag step is used, as projects and events do.
- Pages: `/help-requests/<space id>` (Overview + Chat tabs, web's `HelpRequestDetailPage`),
  `/create/help-request` (web's six steps), `HelpRequestCard` in the feed via the gateway
  index (`kind: help-request`, `neededAt`, signups and slots needed).

## Consequences
- **Integrity change: the project network resets** when this merges (test data only; directory
  and conversation keep their hashes; check with `hc dna hash`).
- Help requests come with ratings, check-ins and history for free (activity history), which web
  doesn't show for them; the UI leaves them out.
- When channels/communities land, help requests get web's required scope tag.

## Sharing (2026-10-31)
web's Share menu sits left of the vote strip for signed-in people (`HelpRolesDock`, with
`ShareUserMenu`'s share-outside half in `ShareOutside.svelte`: Share via…, Copy link, links to
WhatsApp/Telegram/Email/X/Facebook). Sending to a person does what web-backend's
`share_help_request_with_user` does, without a server:
- The link goes into your direct chat with them (started if there is none) as shares.py's
  text, "Shared a help request: <title>\nOpen help request: <url>" (`domain/share.ts`).
- Their device derives web's `hr-share` notification ("A help request was shared with you.",
  Shares & invites category) from that message in a direct chat, as mentions are derived
  (ADR 0046); it opens the request. Someone could fake one by typing the same text in a direct
  chat; it only links to a page, so nothing is gained by it.
- web's "Sent to <name>" toast is left out: the app has no toast host yet; the sheet closes.
