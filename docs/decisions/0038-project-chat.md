# 0038. Project chat: anyone may post, authors delete, and the chat is polled

Status: accepted (2026-10-19). See docs/plans/M4-talk.md. Builds on [0006](./0006-open-membrane.md)
and [0030](./0030-project-clones-and-creator-seed.md).

## Context
Each project clone gets a chat (web's "# project-chat", stored on web as `comments` with
`subject_type = project`). web lets anyone signed in post in a public project's chat
(`add_comment` checks only that the account isn't paused and can view the project), supports
replies (`parent_id`), lets only the author delete a message (`delete_own_comment`) and has no
editing in project chats. Membership lives in `project_integrity` as `Members` links. A second
integrity zome can't link that crate (duplicate WASM symbols), so a members-only rule would have
had to cite the link action and compare zome and link indices by hand.

## Decision
- **A new zome pair**, `discussion_integrity` / `discussion`, holds talk inside a project. M4a adds
  `ChatMessage { body, reply_to }` and one link type, `ChatMessages` (chat anchor → message).
- **Who may post: anyone in the project's network**, as on web. The membrane is open (ADR 0006),
  so this is "anyone signed in". Integrity therefore reads no membership. If a later milestone
  needs members-only talk, that rule goes in a coordinator check or a cited `Members` action, not
  in a dependency on `project_integrity`.
- **Paused accounts** see web's read-only composer. That is a UI check only: standing lives in
  the directory DNA, which project validators can't read (as in ADR 0033). Enforcing it on the
  network waits for moderation (M5).
- **Integrity checks:** a body is 1–4,000 characters (`sp_core::chat`; web has no cap). A reply
  must cite a valid `ChatMessage` of this zome. Messages are never updated. Only the author
  deletes a message or removes its link, and a chat link must point at a message by the link's
  author, so nobody can list or unlist someone else's message.
- **Deleting** removes the chat link and the entry. The message leaves the chat for everyone, and
  replies to it quote "Deleted message". (web keeps a "Deleted message" bubble in place when a
  deleted comment has replies. Here the reply's quote carries that instead, which saves keeping
  deleted messages in the list.)
- **Live updates: polling, not signals.** The UI calls `list_messages` every 5 s while the chat tab
  is visible and every 45 s otherwise (web's `startVisibilityPoll` settings for project chat).
  Remote signals would need each sender to know every reader, plus cap grants for
  `recv_remote_signal`, and hosted browsers would need the gateway to relay them. Polling has none
  of that, and on full nodes (ADR 0036) the read is local. A read returns the newest 200 messages.
- **One "chat" anchor.** The anchor is chosen by the coordinator, and integrity doesn't check the
  link base, so bucketing the anchor by day when chats grow is a coordinator-only change.

## Consequences
- Adding an integrity zome changes the project DNA hash: a new test network (ADR 0029).
- Anyone can write in any project's chat, the same openness as web. Spam handling arrives with
  reports and moderation (M5).
- A quiet chat still costs one zome call per open tab every 5 s. Signals can be added later
  on top of polling without any change to the data.
