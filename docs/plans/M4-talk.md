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
private read markers and pins (DOMAIN_MODEL.md), web's MessagesPage.

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

## M4b as built (2026-10-19)
ADR 0041. web keeps threads in channels and communities; here they live in the project clone,
which is the thread's scope.
- **Integrity** (`discussion_integrity`): `Thread { title, body }`, `Comment { thread, parent,
  body }`; links `Threads` (anchor → thread), `ThreadComments` (thread → comment), `Votes`
  (thread/comment → voter key, tag up/down). Limits in `sp_core::discussion` (title 200, opening
  post 20,000, comment 4,000). A comment must cite a thread, a reply a comment of the same thread,
  a comment link must sit under its own thread, a vote is cast only as yourself on a thread or
  comment. Nothing is edited; only authors delete (no delete offered, as on web).
  `rules.rs` holds the checks; `lib.rs` the types and dispatch.
- **API** (`discussion` coordinator, `threads.rs`, `comments.rs`, `votes.rs`): `create_thread
  { title, body }`, `list_threads { sort, window, utc_offset_minutes }` (newest 200, ordered by
  `sp_index` like web's channel feed), `get_thread(id)` (thread + first 200 comments oldest first,
  flat with `parent`), `add_comment { thread, parent, body }`, `cast_vote { target, direction }`
  (1 / -1 / 0). Vote and comment links for many targets are read in one batched `get_links`;
  own threads, comments and votes come from the chain (`own::entries`, `own::live_links`).
- **UI:** a Threads tab on the project page (`features/threads/ProjectThreads.svelte`, composed
  by the route because features don't import each other), `CreateThread.svelte` (web's
  CreateThreadPage basics step in `CreateSheet`), `/projects/[id]/threads/[thread]`
  (`ThreadPage.svelte`). Components ported from web: `cards/{ThreadCard,VoteStrip,CountPill}`,
  `discussion/{ThreadOverview,DiscussionPanel,DiscussionComment,CommentComposer}`. Styles:
  `styles/discussion.css` from PORTS; three app-only rules in `additions.css`. `FeedSurface`
  takes an optional `href` (the thread page's surface isn't a link).
- **Tests:** `sp_core::discussion` units; scenarios `tests/scenarios/tests/threads.rs` (a
  non-member starts a thread, comments and replies show at once and to others, bad input rejected,
  votes count once per person and can change or be withdrawn, top/oldest sorts); browser test
  `tests/e2e/threads.mjs`. Layout checked at 360, 390 and 1280 px with mock data.
- **Differences from web, for now:**
  - no channel/community tags (the project is the scope);
  - no share menu (DMs, M4c), reports (M5) or @mentions (M4d);
  - comment links are plain http(s) URL links;
  - the thread page doesn't poll;
  - paused accounts get the read-only composer and no "New thread", but the network doesn't
    enforce it (ADR 0038);
  - web's posts are personal, so they aren't here (ADR 0041).
- **Left:** the seed node's feed index doesn't list threads (the archivist would read every clone's
  thread links, plus a mixed card type in `/index/projects` and the home feed), so threads show only
  in their project.
