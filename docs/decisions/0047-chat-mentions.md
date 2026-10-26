# 0047. @mentions in every chat: ranked by closeness, resolved by name, notified on the mentioned person's device

Status: accepted (2026-10-23). See docs/plans/M4-talk.md "M4d as built, second part". Builds on
[0038](./0038-project-chat.md) and [0046](./0046-notifications-derived-on-device.md).

## Context
web's chat composer offers a mention menu in project, event, thread and post chats
(`LiveChatPanel` `mentionSubjects`): typing `@na` lists matching usernames
(`/users/suggestions`), Enter or a tap inserts `@name `. Every chat body links `@name` to
`/profile/<name>` (`utils/linkifyMessageBody.ts`). web-backend sends the mentioned people a
`mention` notification ("You were mentioned.", linked to `?tab=chat&comment=<id>`), as well as
any `reply` notice. A linked message is scrolled into view and marked (`.highlighted`) until
the viewer writes, which also drops `?comment=` from the address.

In the app a profile's address is its agent key, and usernames aren't unique on a P2P network
(`sp_core::handle`: "the key tag tells two## Decision
- **Text stays text.** A message stores `@name` as typed; nothing new is written, so no zome or
  integrity change.
- **Menu in every chat:** project chats (as on web) and, at the owner's request (2026-10-23),
  direct and group chats too. web-backend already notifies mentions typed there.
- **Ranked by closeness** (`domain/mentions.ts`): web-backend ranks people you follow, then your
  followers, then everyone. The app's circles are the people in this chat (a chat's members,
  or the people a project page knows), then people you follow, then your followers (accepted
  follows only, ADR 0054; added 2026-10-26), then the people in your other direct and group
  chats, then everyone in the directory. Within a circle: the exact name, then names starting
  with the query, then names containing it, alphabetically. 8 at most, yourself left out.
- **Same names:** a name shown twice in the menu carries its key tag (`Bob · 7NFm`), as the
  app does elsewhere. `@name` links to the nearest person with that name by the same ranking;
  a name nobody known has is shown styled but unlinked.
- **Sources:** the directory's people list, your follows (`services/follows.ts`
  `followCircles`) and your chats' members (`services/messages.ts` `mentionPool`), read once per chat view when someone types `@` or a message mentions
  someone, and again after a search finds nobody (a new account can lag).
- **Notifications** are derived on the mentioned person's device (ADR 0046): a message by
  someone else whose `@name`s include your username (case ignored). Project chats give web's
  "You were mentioned." project card, linked to `?tab=chat&comment=<id>`; direct and group
  chats you're still in give web's "Thread" card, "<name> mentioned you in <group>." (or "in a
  chat"), linked to `/messages?conversation=<id>`. Two people with the same name both get it.
- **Linked message:** `?comment=<id>` centres the message in the chat log and marks it
  (`.highlighted`) instead of scrolling to the end; your next message clears the mark and the
  parameter.

## Consequences
- No integrity change; nothing resets.
- A name shared by two people links to only one of them (the nearest). Profile keys in
  mentions would need a richer message format, an integrity change.
- Reading the whole directory scales with the number of people; a name index in the directory
  DNA would replace it.
- Each bell refresh now also reads the newest page of messages in every direct and group chat
  you're in (the unread badge already reads their summaries). Fine at today's sizes; a
  per-chat "mentions of me" call would trim it later.
- Internal-path links (`/projects/…`) in messages came on 2026-10-23 (PR #30); the follow
  circles on 2026-10-26, after follows (ADR 0054).
