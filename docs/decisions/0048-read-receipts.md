# 0048. Read receipts in direct chats: a public `Seen` next to the private read mark

Status: accepted (2026-10-23). Builds on [0043](./0043-direct-messages.md) and
[0045](./0045-group-options-edits-pins.md).

## Context
web marks your own messages in a **direct** chat with ✓ (sent) or ✓✓ (read): web-backend
returns the other member's `conversation_members.last_read_at` as `peer_read_at` with the
thread, and `LiveChatPanel.messageIsRead` compares it with each message's time. Groups show no
receipts. In the app a read mark (`ReadMark`) is a private entry (ADR 0043): it never leaves its
author's chain, so the other person can't see it.

## Decision
- **New public entry `Seen { at, invite }`** in the conversation DNA, linked from the chat's
  `messages` anchor (`LinkTypes::Seens`). Validation: direct chats only, and the author is in
  the chat (the creator, or `invite` is their invitation), like a message.
- **Written with the read mark:** `mark_read` already writes a private `ReadMark` only when
  something from the other person is unread; in a direct chat it now also writes a `Seen` with
  the same `at`. So a receipt costs one entry per "caught up", not one per poll.
- **Read:** `peer_read_at()` takes the other person's newest `Seens` link and fetches only that
  record. The conversation view calls it with every load of the chat (as web reads it with the
  thread) and marks your own sent messages ✓ or ✓✓ (`chatFormat.ts` `withReceipts`; web's
  `.receipt` styles were already ported).
- **The private `ReadMark` stays** for your own unread counts, which are read from your own
  chain and so count at once (CLAUDE.md: own data comes from your own chain).

## Consequences
- **Integrity change** (conversation DNA only): the hApp version changes, hosted apps
  reinstall, and existing direct and group chats are left on the old network (test chats
  only, as with ADR 0045). Projects and people are untouched.
- A direct chat's members (only they know its seed) can see when the other person last caught
  up, as on web. There is no setting to turn receipts off; web has none either.
- `at` is the reader's own claim (their clock, or the newest message's time if that is ahead);
  a wrong one only shows wrong ticks to the other person.
