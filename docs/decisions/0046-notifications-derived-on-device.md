# 0046. Notifications are derived on each device from the projects it has opened

Status: accepted (2026-10-21). See docs/plans/M4-talk.md "M4d". Builds on
[0044](./0044-public-chats-list.md); DOMAIN_MODEL.md "Not on the DHT".

## Context
web's Notifications page lists rows web-backend writes into a `notifications` table when
something happens (`create_notification`), each with a kind, an actor, a title, a body, a link
and a read flag; the bell in the top bar and bottom nav shows the unread count. Of web's kinds,
the app has the activity behind three so far: `prj-phase-vote` (the other members, when a
phase move opens: "Vote on advancing to …"), `prj-phase-done` (the other members, when a move
or a return passes: "The project phase changed to …") and `reply` on a project chat (the
replied-to author, "Replied to your comment", else the project's owner, "Replied to your
project"). `prj-plan-lead` is off by default on web (`DEFAULT_NOTIFICATION_CATEGORIES`) and the
app has no notification settings, so it is left out. Follows, shares, invites, roles, bot
reviews and mentions have no app feature yet.

The domain model already says notifications are not stored centrally. A project's activity
lives in its own network (a clone), and reading a project not yet opened joins its network.

## Decision
- **Derived, not stored:** each device builds the list from the project clones on its node
  (`clonedSeeds`, as the Public tab does): per project `get_project`, `list_proposals` and
  `list_messages`, turned into web's cards by `domain/notifications.ts` with web-backend's
  recipients and wording (no card for your own actions; phase cards only while you're a
  member). Card times are the proposal's opening, its outcome, or the message.
- **The bell** is refreshed by the app shell every 30 s while the page is visible
  (`stores/notifications.svelte.ts`); the page re-reads on open.
- **Read marks stay in the browser**, per person (`sp.notificationsRead.<agent>`): everything up
  to a time (`Mark all read`) plus single cards read since. The first look on a device sets the
  time to now: web only notified the people who were members when something happened, which a
  device can't tell for the past, so older activity shows as read.
- **Links:** a card opens its project's page, found from the seed with a new directory
  coordinator call, `find_listing(seed)` (seeds name their creator, so only that person's
  listings are read). Like web: replies open `?tab=chat&comment=<id>`, phase votes
  `?open=vote&voteKind=phase_change&voteTarget=<id>`, which opens the Vote sheet on that proposal
  and drops the link when the sheet closes.

## Consequences
- No integrity change (`find_listing` is coordinator-only), so nothing resets.
- Read marks don't follow you to another device or browser, as for the Public tab; they move
  to private entries with the next directory-DNA integrity change.
- Only projects opened on this node notify. Every project you created, joined or wrote in was
  opened here, but a restore on a new device starts with none until they're opened again.
- Each refresh reads three zome calls per opened project. Fine at today's sizes; signals or a
  per-project summary call would trim it later.
- A chat card opens the Chat tab but doesn't scroll to the message yet (web highlights it).
- Not yet: web's other kinds (each comes with its feature), notification settings, push.
