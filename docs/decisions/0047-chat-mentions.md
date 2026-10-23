# 0047. @mentions in project chats resolve by name, and notify on the mentioned person's device

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
(`sp_core::handle`: "the key tag tells two people apart").

## Decision
- **Text stays text.** A message stores `@name` as typed; nothing new is written, so no zome or
  integrity change.
- **Menu:** project chats only, as on web. Suggestions are the directory's people list
  (`list_people` + names, read once per page when someone types `@` or a message mentions
  someone; an empty search reads it again, since a new account can lag), matching anywhere in
  the name, yourself left out, 8 at most.
- **Links:** `@name` links to the profile of someone in this chat with that name first, else
  the first in the directory list; a name nobody known has is shown styled but unlinked.
- **Notification:** derived on the mentioned person's device (ADR 0046): a project-chat message
  by someone else whose `@name`s include your username (case ignored) gives web's `mention`
  card. Two people with the same name both get it, as both would read it as theirs.
- **Linked message:** `?comment=<id>` centres the message in the chat log and marks it
  (`.highlighted`) instead of scrolling to the end; your next message clears the mark and the
  parameter.

## Consequences
- No integrity change; nothing resets.
- A name shared by two people links to only one of them; the menu shows both names alike, as
  web's does (web's usernames are unique). Profile keys in mentions would need a richer message
  format, an integrity change.
- Reading the whole directory scales with the number of people; a name index in the directory
  DNA would replace it.
- Not yet: mentions in direct and group chats (web-backend notifies those when typed, but web
  offers no menu there; the app would also have to read every chat on each bell refresh), and
  web's internal-path links (`/projects/…`) in messages.
