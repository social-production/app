# 0101. Photos on posts ride on the post's chat, with no integrity change

Status: **accepted** (2026-11-13; the owner asked for image posts "like Reddit/Twitter" in
`docs/plans/UX.md` and said "go for it" on 2026-11-13). Builds on ADR 0056 (photos and files
in the project chat), ADR 0070 (a post is a commons space whose chat holds its comments) and
ADR 0100 (the post page).

## Context
A post's charter has no place for attachments, and adding a field would change the project
DNA's hash: a new network, every project lost. But a post's space already has a chat, and
that chat already carries photos as 1 MB chunk entries listed by a `ChatMessage`
(ADR 0056). The `ChatMessages` link that lists a message in its space has a free tag.

## Decision
- **A post's photos are one chat message by the post's author**, with no text and one to
  four photos (`media::PHOTOS_MAX`, Twitter's four), listed in the space's chat with the
  link tag `media`. Coordinator only: the entry, its chunks and the link are the chat's own
  types, so every validation rule of ADR 0056 still applies and **the DNA hashes do not
  change** (checked against the live seed's `/readyz`).
- `discussion.set_post_media(attachments)`: only the space's founder, only photos, only
  once. `discussion.post_media()` returns the founder's media message's attachments (the
  caller's own chain first, so the author sees them at once).
- **A media message is not a comment:** `list_messages` skips links tagged `media` (from
  anyone) and the caller's own media messages, so comment lists and the index's comment
  count leave it out.
- UI: the create form picks up to four photos (resized to fit 1 MB as chat photos are) and
  writes them after the post is created. `PostPhotos.svelte` shows them in a rounded grid
  under the text, on the post's page (a photo opens full screen) and on the feed row. It
  reads when the row scrolls into view and keeps each photo for the session.

## Consequences
- No reset. An older client would show the media message as an empty comment with photos;
  there are no older clients (the UI deploys with the seed).
- Photos are fixed at creation: there is no editing a post, so none for its photos either.
- If writing the photos fails after the post is created, the post stands without them.
- Anyone can write a `media`-tagged chat message in a post's space. It is ignored unless its
  author is the founder, and it never shows as a comment. Harmless, but it is a way to store
  bytes nobody sees, as an unlisted chat message already is.
- Each post row in a feed makes one extra read for its photos (most return none). If feeds
  get slow, the index can carry a "has photos" flag (a gateway change, no reset).
- Threads, projects and events could take photos the same way; not built (not asked for).
