# 0045. Group options, edits and pins: cited authority, replayed membership

Status: accepted (2026-10-20). See docs/plans/M4-talk.md "M4c". Builds on
[0043](./0043-direct-messages.md) and [0044](./0044-public-chats-list.md).

## Context
web's Messages page lets the group's creator rename it, add and remove people; anyone can leave,
and when the creator leaves "the longest-standing member takes over managing it"
(web-backend `conversations.py`). People edit their own messages; either person in a direct
chat, or the group's manager, pins up to 5 messages (`attachments.py`). Each person can pin,
mute or delete a chat in their own list (`set_conversation_preferences`); a deleted direct chat
comes back when a new message arrives, and deleting a group leaves it.

ADR 0043 fixed the creator in the chat's seed and let only them invite. Managing can now pass
to someone else, and people can be removed, but validators still can't look up a chat's history:
each write is checked on its own.

## Decision
- **Authority is cited.** Manager writes (`Invite`, `Title`, `Removal`, a group `Pin`, and a
  `Leave` that hands over) carry `by`: `None` for the creator, or the `Leave` in which the
  previous manager named the author as successor. A valid `Leave` was itself written by a
  manager, so one step of the chain is enough. Membership is cited the same way: a `Message`, a
  `Leave` or a direct-chat `Pin` cites an `Invite` naming its author (any manager's).
- **What counts is replayed** (`sp_core::roster`): invitations, removals and leaves in time
  order, from the creator. A manager write by someone who had already handed over is valid but
  doesn't count; a message written while its author was out (removed or left, not yet invited
  back) is not shown; names and pins count only from whoever had the right at the time. A
  leaving manager's coordinator names the longest-standing member; readers fall back to that
  rule if the named one is gone.
- **Edits** are `Edit { message, body }` entries by the message's author, listed under the chat
  anchor; the newest shows, marked "edited". Messages themselves still never change (no
  `Update` ops), so reads stay batched.
- **Pins** are `Pin { message, pinned }` entries; the newest that counts decides. At most 5
  (coordinator rule, web's `MAX_PINS`).
- **List settings** (pin, mute, delete) are a private `Prefs` entry, newest wins, like
  `ReadMark`. Delete hides the chat until something newer arrives. For a group, Delete leaves
  it. Project chats in the Public tab keep these settings in the browser, like their read marks
  (ADR 0044).
- **Leaves show as a line** ("Ann left the group."), made by readers from the `Leave` entries
  rather than written as a message, so the UI names the person from the directory.
- The UI follows web: the chat menu (row right-click / long press, or the chat header), the
  Chat info sheet, the pin bar, and Edit and Pin in the message menu.

## Consequences
- Integrity change in the conversation DNA only: new hApp integrity version, hosted apps are
  reinstalled (ADR 0028/0029). Existing direct and group chats are on the old network and are
  not carried over (the test network has only test chats).
- Removed people can still technically write to the network (validators can't see the removal);
  everyone's reads drop those writes, and their own app refuses. The seed stays known to them,
  so they can still read what is gossiped, as ADR 0043 already says of anyone holding a seed.
  Rotating to a new chat clone on removal would close that; it isn't worth it before a
  membrane proof exists.
- Differences from web: deleting a group from your list also posts "left the group" and hands
  over management (web's Delete drops the membership silently, leaving a group without its
  creator); the project chat's header menu has Mute only (no "Open page": the Public tab knows a
  project by its seed, not its listing); there is no toast after leaving.
- Clock skew: membership changes are ordered by their authors' timestamps. A message sent just
  as its author was removed may show or not depending on clocks; both are acceptable.
