# 0044. The Public tab lists your projects' chats from this node, with read marks kept on the device

Status: accepted (2026-10-20). See docs/plans/M4-talk.md "M4c". Builds on
[0038](./0038-project-chat.md) and [0043](./0043-direct-messages.md).

## Context
web's Messages page has a second tab, Public, listing the chats of the projects and events you
belong to (web-backend `linked_chats.py`: projects you're a member of, plus any you commented
in), each with its last message and an unread count. web keeps a per-person `last_read_at` for
each chat (`subject_chat_reads`), set from Messages and from the project's own Chat tab
(`LinkedChatReadMarker`). The app has no events yet, so only projects apply.

Two things web gets from one database are spread out here. Membership and messages live in
each project's own network (a clone, ADR 0030), and reading a project you haven't opened clones
it, which joins its network. And the project DNA has no private read marks: adding them is an
integrity change, which resets every project again.

## Decision
- **Which projects:** the project clones already on this node (`holo/cells.ts` `clonedSeeds`),
  read by seed straight from their cells, keeping those where you're a member or have written a
  message. Every project you created, joined or wrote in was opened on this node, so nothing new
  is joined just to fill the list. The directory isn't consulted: listing every project costs a
  read per listing, and a new project's listing reaches others' `list_projects` late. Rows and
  read marks are keyed by network seed (`?project=<seed>`).
- **What each row shows:** the project's title and member count (`get_project`) and the last
  message of its chat (`list_messages`), read local-first. Rows sort by last message, else the
  project's last activity; the list refreshes every 8 s like the Personal list.
- **Opening a row** shows the project chat inside Messages with web's header,
  "Project chat · N members". It is the same chat as the project's Chat tab.
- **Read marks stay in the browser for now:** the newest message seen, per person and project,
  in `localStorage` (`sp.projectChatRead.<agent>.<seed>`), written from Messages and from the
  project's Chat tab while it shows. The
  unread count is other people's messages after it.

## Consequences
- No integrity change: the DNA hashes stay, so nothing resets.
- Read marks don't follow you to another device or browser, and clearing site data makes every
  project chat unread again. They move to private entries when the project DNA next changes
  integrity for another reason (or to the conversation inbox if that changes first).
- After a restore on a new device the list starts empty, until projects are opened again; the
  restore doesn't recreate project clones yet.
- Each refresh reads every listed project's chat page (up to 200 messages). Fine at today's
  sizes; a per-chat summary call in the discussion coordinator would trim it later.
- Not yet: web's members panel behind the member count, pins and mutes (the chat menu), events
  and help requests.
