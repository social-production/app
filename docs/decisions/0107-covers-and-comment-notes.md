# 0107. Profile covers and comment notes in the directory

Status: **accepted** (2026-11-16). The owner approved the Profiles and Comments mockups
(https://claude.ai/artifact/9N8yzimwCoGkWXpUr3Lp4C) and left the network choice to Claude,
who chose to ship this in the same directory reset as ADR 0106 (#163), so the network is
reset once.

## Context
The owner asked for a cover photo that you change from your own profile, and for "commented"
rows: when someone you follow comments on a thread, a post or in a project's chat, Home shows
it as their comment, not as something they made; their profile lists it too.

A comment is a message in its space's own clone of the project DNA. Reading someone's
comments that way would mean joining every space they have written in. A profile record is
read for every avatar in every feed, so a cover inside it would travel with every read.

## Decision
1. **`Cover { mime, bytes }`**, its own directory entry (JPEG, PNG or WebP, at most 512 KB),
   linked `AgentToCover` from its owner's key. Setting a new one deletes the old link and
   entry; `None` removes it. `get_cover(agent)` reads it; your own comes from your chain.
2. **`CommentNote { space, message, reply_to, excerpt }`**, linked `AgentToComments` from
   its author's key: the space's listing id, the message id in that clone, the person a
   reply answers, and up to 280 characters of the comment. The UI writes it right after the
   comment lands and takes it back when the comment is deleted (`unnote_comment`).
   `comments_of(agent)` returns the newest 50. The feed row reads the space's title and
   type from its listing, so the note holds no copy of them.
3. Both link types, like the profile's, may only be written from the author's own key.
   Covers and notes may be deleted by their author.

## Consequences
A directory network reset (with #163's). An edited comment keeps its first words in its
note. A note can outlive a comment if the deletion's second write fails; the row then opens a
space where the comment is gone, which the space already shows as "deleted". Scenario
`notes` covers both records.
