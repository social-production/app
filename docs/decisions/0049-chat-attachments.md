# 0049. Photos and files in direct and group chats: 1 MB chunk entries named by the message

Status: accepted (2026-10-24). Builds on [0043](./0043-direct-messages.md); settles
DOMAIN_MODEL.md's open item 3 ("Files (`blobs`)") for chats.

## Context
web's chats take photos and files (`LiveChatPanel` `allowAttachments`, web-backend
`services/messages/attachments.py`): photos are JPEG/PNG/WebP resized in the browser to fit
1 MB (`compressChatPhoto`), files up to 10 MB, no videos, at most 10 per message, and a photo
or file may be sent without text. The list row and quotes then read "Photo" or the file's name
(`_message_preview`). Holochain caps an entry at a few MB, and every entry is gossiped to its
DHT authorities, so a 10 MB file can't be one entry.

## Decision
- **`Chunk { bytes, invite }`**, a public entry in the conversation DNA holding at most 1 MB
  (`sp_attachment::CHUNK_MAX`). Its author must be in the chat (`invite`, as on a message).
  The sender writes each part with `upload_chunk` (one call per part: a source chain takes one
  write at a time), then the message.
- **`Message.attachments: Vec<Attachment { kind, name, mime, size, chunks }>`**: web's
  `message_attachments` row plus the parts in order. Validation applies web-backend's rules
  (`sp_attachment::check_attachment`), checks the number of parts against the size, and checks
  that every part is a `Chunk` by the message's author (`must_get_action`: the part's own
  validation checks its size and membership). The body may be empty beside attachments.
- **Reading:** messages carry the descriptions; the UI fetches parts with `get_chunk` (local
  first, then the network), joins them into a `Blob` and shows an object URL, cached per chat
  view. Photos load as their bubbles render; a file is read when its bubble is tapped, then
  downloaded. Deleting a message also deletes its parts.
- **UI ported from web:** the composer's photo and file buttons, previews and errors
  (`attachmentLimits.ts` copied), the photo row, file bubbles and footers in `ChatBubble`, and
  `PhotoViewer` (styles from PORTS).
- **Rules in their own crate, `sp_attachment`:** adding a module to `sp_core` changed the
  directory and project DNAs' hashes too (every integrity zome links `sp_core`), which would
  have reset everyone's people and projects. Only the conversation DNA links `sp_attachment`.

## Consequences
- **Integrity change** (conversation DNA only): the hApp version changes, hosted apps
  reinstall, and existing direct and group chats are left on the old network (test chats).
  Directory and project DNA hashes are unchanged (checked with `hc dna hash`).
- **Project chats have no attachments yet.** web allows them there too; that needs a project-DNA
  integrity change (a reset of projects), so it waits for the next one, with the project-chat
  read marks (STATUS.md Next).
- Anyone holding the chat's seed can fetch its parts, as with its messages (ADR 0043's privacy
  trade-offs). Parts are not encrypted beyond that; a gateway's operator can read hosted
  people's files, as web's server can.
- A 10 MB file is ten DHT entries, each stored by the chat's authorities; deleting a message
  marks its parts deleted, but peers that held them may keep the bytes.
- Very large chats of photos load every photo on view; a lazy loader can come later if needed.
