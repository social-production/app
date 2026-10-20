# M4 — Talk (plan, 2026-10-19)

People talk where the work happens: a chat in every project, discussion threads, direct
messages, and in-app notifications. The reasoning for the chat is in ADR 0038.

## M4a — project chat (this batch)
- **project DNA, new `discussion` zome** (integrity + coordinator), separate from
  `project_integrity` so talk can change without touching membership and governance rules:
  - `ChatMessage { body, reply_to }`: 1–4,000 characters (`sp_core::chat`); `reply_to` cites
    another chat message (web's `parent_id`).
  - `ChatMessages` link: the "chat" anchor → a message, made by the message's author.
  - Never edited (web's project chat has no edit). Only the author deletes; deleting removes the
    link and the entry.
- **API:** `post_message { body, reply_to }`, `delete_message(id)`, `list_messages()` → the
  newest 200, oldest first, each with its reply quote (author, first 120 characters, or
  "deleted"). Reads merge the network with the caller's own chain, so your own message and
  deletion show at once.
- **Who may post:** anyone signed in, as on web (open membrane, ADR 0006).
- **Live updates:** the UI polls every 5 s while the chat tab is visible, every 45 s otherwise.
- **UI from web:** ProjectChatTab and LiveChatPanel (text, day dividers, author runs, reply
  quotes with jump-to-original, the bubble menu with View profile / Reply / Copy / Delete, the
  growing composer, Enter to send on desktop), ConfirmHost for "Delete this message?", and the
  immersive phone layout (top bar and bottom nav hide while the chat is open).
- **Tests:** `sp_core::chat` units; scenarios `tests/scenarios/tests/discussion.rs` (a non-member
  posts and reads, replies carry quotes, own writes show at once, blank/long/bad-reply rejected,
  only the author deletes and the reply then quotes "Deleted message").

## M4b — discussion (next)
`Thread`, `Post`, `Comment`, `Reaction` in the same `discussion` zome (web's threads, posts and
their comments and votes), with the feed index picking up threads.

## M4c — direct messages
The conversation DNA (clone per chat, listed participants only), `Message` and attachment refs,
private read markers and pins (DOMAIN_MODEL.md), web's MessagesPage. First part as built below
("M4c as built").

## M4d — notifications
Derived on each device from signals and new links (DOMAIN_MODEL.md "Not on the DHT"), including
chat mentions and replies.

## M4a as built (2026-10-19)
- `dnas/project/zomes/{integrity,coordinator}/discussion`, registered in `dnas/project/dna.yaml`.
  Holochain routes a delete op to the zome of the deleted entry, so `project_integrity`'s "never
  deleted" rule doesn't touch chat deletes.
- Reads fetch message records local-first in one batched `get` (messages never change), then
  the network for what's missing. Quotes for originals outside the page use `get_details` to
  tell "older" from "deleted".
- UI: `components/chat/{ChatPanel,ChatBubble,ChatComposer}.svelte` with `chatFormat.ts`
  and `chatViewport.ts`; `components/overlay/ConfirmDialog.svelte`;
  `features/projects/{ProjectChat.svelte,chatState.svelte.ts}`; the Chat tab in ProjectDetail
  (mounted on first open, as web lazy-loads it; the action dock hides on that tab, as on web);
  `stores/chatChrome.svelte.ts` for the immersive phone layout. Styles: `styles/chat.css` and
  `styles/confirm.css` from PORTS.
- Differences from web, for now: no photos or files (blob storage is an open question,
  DOMAIN_MODEL.md), no @mentions (people search arrives with notifications), no reports (M5),
  no unread marker or linked-chats list (M4c/M4d). Links in messages are plain URL links; web's
  internal-path and mention links come with those features. A deleted message with replies leaves
  no "Deleted message" bubble; its replies quote "Deleted message" instead (ADR 0038). Paused
  accounts get the read-only composer, but the network doesn't enforce it yet.

## M4c as built (2026-10-20, first part; ADR 0043)
- `dnas/conversation/zomes/{integrity,coordinator}/messages`, registered in
  `dnas/conversation/dna.yaml`; seed and limit rules in `sp_core::conversation`.
- Chat clone API: `start_chat { members, title }` (creator only), `post_message`,
  `delete_message`, `list_messages` (the project chat's shape), `summary` (kind, creator,
  members, group name, last message, unread), `mark_read`. Inbox (base cell): `send_invite
  { to, seed }`, `inbox()`.
- UI: `features/messages/{MessagesPage,ConversationView}.svelte` + `inboxState.svelte.ts`;
  `components/messages/{ComposeMessageSheet,SuggestionList,ChatRow}.svelte`; the profile's
  Message button (`features/people/MessageButton.svelte`); `services/messages.ts`;
  `holo/cells.ts` `conversationCell`/`newChatSeed`. The chat state moved to
  `components/chat/chatState.svelte.ts` with a `ChatSource`, shared with the project chat.
  Styles: `styles/messages.css` (PORTS: MessagesPage, ComposeMessageSheet, SuggestionList,
  PageHeader).
- Tests: `sp_core::conversation` units; scenario `tests/scenarios/tests/messages.rs`.
- Not yet (next M4c parts, from web): the Public tab (project chats in the list), group options
  (rename, add/remove members, leave), the chat menu (pin/mute/delete chat), message edits,
  pins, attachments, calls, "seen" receipts.
