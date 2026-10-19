# 0041. Discussion threads live in project clones; votes are links; posts wait

Status: accepted (2026-10-19). See docs/plans/M4-talk.md. Builds on [0038](./0038-project-chat.md),
[0006](./0006-open-membrane.md) and [0012](./0012-local-index.md).

## Context
On web, a thread (`threads`: title ≤ 200 characters, an unbounded opening post) is tagged to one
or more channels or communities, not to a project. Its comments share the `comments` table with
the project chat (`subject_type = thread`, `parent_id` replies in the same thread, the first 200
oldest first, nested in the response). Threads and comments take up/down votes
(`content_votes`: one row per voter and target, direction 1 / -1, "neutral" deletes it; the score
is ups minus downs). Anyone signed in may start a thread, comment or vote. web has no edit or delete
for threads or thread comments (the comment delete endpoint is used by the chat only). Channel feeds
sort threads by trending (votes capped at 40 plus comments capped at 30, decayed by age) or recent,
within a time window; `last_activity_at` is set at creation and never moved.

web's project page has no thread list. Channels and communities have no home here yet
(DOMAIN_MODEL.md, open question 2: likely project-DNA clones with another `kind`). Posts (`posts`)
are personal: written to the author's followers or the public, not to a project.

## Decision
- **Threads live in the project clone**, in the `discussion` zome pair beside the chat (ADR 0038).
  The project is the thread's scope, where web uses channel tags. When channels or communities
  become clones of the project DNA, they get threads with no new code.
- **Entries:** `Thread { title, body }` and `Comment { thread, parent, body }`. Limits in
  `sp_core::discussion`: title 1–200, opening post 1–20,000 (`LONG_TEXT_MAX`; web has no cap),
  comment 1–4,000 (the chat's cap, since web stores both in one table). Integrity checks that a
  comment cites a `Thread` and that a reply cites a `Comment` of the same thread.
- **Links:** `Threads` (the "threads" anchor → a thread) and `ThreadComments` (thread → comment),
  each made by the entry's author and pointing at it, as with chat links. Integrity also checks that
  a comment is listed under its own thread.
- **Votes are links, not entries** (DOMAIN_MODEL.md's `Reaction`): `Votes`, from the thread or
  comment to the voter's own key, tag up (1) or down (2). Integrity checks the voter is the link's
  author, the tag, and that the base is a `Thread` or `Comment`. Changing a vote deletes the old
  link and adds a new one; "neutral" just deletes. "One live vote per person" can't be checked in
  validation (it would need the whole link set), so reads count each voter's latest link only.
- **Nothing is edited** (web has none). Only an entry's or link's author can delete it, the same
  rule as the chat, but the coordinator offers no thread or comment delete because web's UI has none.
  The integrity rule is in place so adding author deletes later is a coordinator change.
- **Reads:** `list_threads { sort, window }` reads the newest 200 threads (the caller's own from its
  chain), batches the vote and comment links in one `get_links` call each, and orders them with
  `sp_index` (the same ranking as the project feed, `ItemKind::Voted`). `get_thread` returns the
  thread and its first 200 comments, oldest first, flat with `parent`; the UI nests them. The
  caller's own comments and votes come from its chain, so they show at once.
- **UI:** a Threads tab on the project page (the thread list, laid out like web's channel feed, with
  sort, window and "New thread"), and `/projects/[id]/threads/[thread]` (web's ThreadDetailPage).
  The page loads on open and after each write; it doesn't poll (web reloads on navigation too).
- **Posts are left out.** web's posts belong to their author and their followers, not to a
  project, so they go with the follower feed in the directory DNA (or a person's own space), not
  in a project clone. They need their own placement decision.
- **The feed index doesn't pick up threads yet.** The seed node's index (ADR 0031) lists projects
  from the directory DNA; listing threads means the archivist reading every project clone's thread
  links and a mixed card type in `/index/projects` and the home feed. That is its own change.

## Consequences
- New entry and link types change the project DNA hash: a new test network (ADR 0029), as with M4a.
- A popular thread costs one batched link read per list; with sharding (ADR 0037) the counts may
  come from fewer peers, which is acceptable for display numbers.
- Two live votes from one person (two devices voting at the same moment) count once; the older
  link stays on the DHT until that person votes again.
- Threads have no report or moderation path until M5, the same as the chat.
