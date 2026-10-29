# M5 — Work (plan, 2026-10-25)

What people do together, ported from web: follows, activities and roles in projects, events,
help requests, communities and channels. Ordered so each DNA is reset once per batch: every
integrity change starts a new network for that DNA (CLAUDE.md), and the owner has said the
test network's data is temporary until the FastAPI site is replaced (STATUS "Owner direction").
Every batch ports web's screens (PARITY.md lists what each one unblocks) and ends deployed with
scenario + browser tests, as M1–M4 did.

## M5a — directory batch: follows and feedback (one directory reset) — built (ADR 0054)
- **Follow** (planned in DOMAIN_MODEL since M1, never built): a `Follows` link agent → agent,
  made by the follower; unfollow deletes it. Web's private profiles need follow requests
  (`follow-request` / `follow-accepted`): a request link the target accepts by linking back.
- **Feedback** (web's Feedback page, left-rail button): a `Feedback` entry + votes, DOMAIN_MODEL
  says directory.
- **Unblocks** (PARITY.md): profile followers/following counts, Follow button and people sheet;
  mention ranking by follows (ADR 0047); the feed's Home scope and the Personal feed; Settings
  Follow requests and Privacy; follow notifications.
- **Reset:** the directory DNA hash changes, so people sign up again on the test network
  (hosted apps reinstall, ADR 0028). Projects and chats keyed to old agents become unreachable.
  Say so in STATUS and README when it ships.
- Decided in ADR 0054: requests exist (approval switch). Mention ranking by follows (ADR 0047
  update) and the Home scope with the personal feed (ADR 0055) followed on 2026-10-26.

## M5b — project batch: activities and roles (one project reset) — first part built (ADR 0056)
- `Activity`, `Role`, `Assignment`, `Rating` (DOMAIN_MODEL), web's
  `ActivitySchedulingPanel`, `ActivityAgendaList`, `CollapsibleActivityCard`,
  `ActivityCreationWizard`, and the activity cards in the feed.
- **Ride along** (they were waiting for the next project-DNA change): project-chat photos and
  files (ADR 0049), private project-chat read marks and list settings (ADR 0044).
- Unblocks: the right rail's activity, feed activity cards, the display-timezone setting,
  `roles` notifications.

## M5c — events and help requests — public events built (ADR 0057)
- Decide DOMAIN_MODEL open question 1 in an ADR. The leaning: clones of the project DNA with a
  `kind`, reusing membership, governance and plans; their own listings in the directory.
- Web's events feature is large (~4,200 lines across `features/events/**`); port it in parts
  (detail page, lifecycle phases, create flow, cards), as M3 did for projects.
- Help requests (`features/help-requests`, ~870 lines) after events.

## M5d — communities and channels (scopes) — ADR 0068 proposed (2026-10-29)
- Decided by ADR 0065 + 0068: public channels/communities are commons spaces (modes `channel`,
  `community`), closed communities are clones; handles claimed first come first served in the
  directory; tags `#channel` / `@community`; scope proposals reuse the project DNA's. Parts:
  (a) DNA + scenarios (one project reset), (b) create flows, scope pages, join,
  left rail, Discover, (c) tagging + tag icons, (d) closed communities + invites, (e) settings
  and votes. Web: `features/communities`, `channels`, `shared-directory`, the left rail's lists,
  scope tags with icons on cards, private (closed) communities hiding their items.
- Unblocks: the card tag icons, the left rail, "Join channels / communities" copy, the scope
  menu's options.

## Not in M5
Map and places, moderation and the ⋮ report menu, push relay (M6); mobile and multi-seed (M7).
