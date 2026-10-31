# 0070. Threads and posts are commons spaces; comments are their chat, with votes

Status: **accepted** (2026-10-31). Follows [0065](./0065-public-commons-network.md),
[0067](./0067-help-requests.md) and [0068](./0068-channels-and-communities.md). Parity item
"Posts and threads" (`docs/plans/PARITY.md`).

## Context
web has two standalone discussion items (NOT tabs on project or event pages; STATUS "Owner
decisions" keeps `claude/m4b-threads` parked):
- **Threads** (`create/thread`, `threads/[slug]`; web-backend `services/content/threads.py`):
  title (≤ 200), body, at least one channel or community tag, up/down votes, comments. They
  appear in the public feed (`ThreadCard`) under their tags.
- **Posts** (`create/post`, `posts/[id]`; `services/content/posts.py`): a body only, audience
  `public` from web's create page, up/down votes, comments. They appear on personal feeds
  (yours plus people you follow) and profiles (`PersonalPostCard`), not the public feed.
- Both pages share `DiscussionPanel`/`DiscussionComment`: comments with nested replies, each
  with an up/down `VoteStrip`, sorted oldest / newest / top, and a composer.

## Decision
- A thread or post is a **commons space whose charter mode is `thread` or `post`** (subtype
  `standard`), checked by a new crate `sp_discussion` so `sp_core` and the other DNAs keep their
  hashes. Like a help request it has no lifecycle: the project coordinator gives it no phases
  (stage "Thread" / "Post"), the governance coordinator refuses proposals, the activities
  coordinator refuses activities.
- A thread's charter: title, body as description, its channel/community tags. A post's
  charter: the body as description, and as title the body's first line cut to 80 characters
  (web's search index title; posts show no title).
- **Votes on the item** are the space's Support/Oppose signal (as help requests, ADR 0067);
  the count is support minus opposition.
- **Comments are the space's chat** (discussion zome): web's `parent_id` is the message's
  `reply_to`, so replies nest. **Comment votes** are a new `MessageVotes` link in the discussion
  integrity zome: message → the voter (only as themselves), tag `[UP]` or `[DOWN]`, only on a
  chat message. Changing a vote deletes your old link first; reads count each person's newest
  link once. Coordinator calls `vote_message` and `message_votes` (the panel asks for one page's
  comments, at most `PAGE_MAX`).
- Feed: the gateway index already carries every listed space; posts are kept out of the public
  feed and appear only when the request names creators (Home's personal rows, profiles).
  Threads and posts rank by votes, as on web.
- Pages: `/threads/<space id>` and `/posts/<space id>` (web uses a slug for threads; the app's
  ids are hashes, as for help requests), `/create/thread`, `/create/post`, `ThreadCard` in the
  public feed and `PersonalPostCard` on personal rows.

## Consequences
- **Integrity change (discussion zome and charter modes): the project network resets** when
  this merges (test data only; directory and conversation keep their hashes).
- Project, event and help-request chats keep their flat chat UI; the vote link could serve
  them later but nothing shows it there (web doesn't).
- **New members' cap:** every directory listing counts against the new-member cap of 2 per
  7 days (`sp_governance::projects`, enforced by directory integrity), so a new member's posts
  and threads share it with their projects, events and help requests. web lets anyone who can
  participate post freely. Lifting it for posts means a directory integrity change (an account
  reset), so it waits for the owner's review (STATUS "For the owner to review").
- Comments beyond the chat's newest 200 aren't shown (web's `get_comments` also caps at 200).
- Moderation (⋮ Report, hidden comments), linked subjects in post bodies (`LinkedPostBody`)
  and "Following + popular" discovery rows come later with their own parity items.
