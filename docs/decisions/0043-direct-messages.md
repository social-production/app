# 0043. Direct and group chats: a clone per chat, invitations through an encrypted inbox

Status: accepted (2026-10-20). See docs/plans/M4-talk.md "M4c". Builds on
[0005](./0005-dna-topology.md), [0006](./0006-open-membrane.md),
[0030](./0030-project-clones-and-creator-seed.md) and [0038](./0038-project-chat.md).

## Context
web's Messages page lists direct chats (two people) and named group chats. web-backend keeps
them in `conversations` and `conversation_members`: the creator starts a chat, only members
read and write, only a group's creator renames it or manages members, a direct chat with the
same person is reused, and each member has a private `last_read_at`. ADR 0005 put chats in the
conversation DNA, cloned per chat, private to its participants. Two things are new compared
with projects: the other person has to learn that a chat exists, and nobody else should.

## Decision
- **One clone per chat**, seeded `<creator key>.<direct|group>.<nonce>` (`sp_core::conversation`).
  The seed names the creator, as a project seed does (ADR 0030), and its kind. The random part is
  24 bytes: the seed is the chat's secret address.
- **Who writes:** the creator, and people the creator invited. An `Invite { agent }` entry may
  only be written by the creator (web: only the creator manages members). Every `Message` cites
  its author's invitation, and validation checks that the cited invitation is the creator's and
  names the author; the creator's own messages cite none. Validators need no lookups, only the
  cited record. A direct chat takes one invitation (coordinator rule); a chat has at most 50
  people.
- **Group names:** `Title` entries by the creator, newest wins. Direct chats have none; the list
  shows the other person's name, as web does.
- **The inbox is the uncloned base cell** of the conversation role, which everyone already has.
  An invitation is an `Inbox` link on the invited person's key, pointing at the sender's key,
  whose tag is the chat's seed encrypted from sender to recipient
  (`ed_25519_x_salsa20_poly1305_encrypt`, keys stay in lair). Only the recipient can read which
  chat it is. The creator sends one to themselves too, so their other devices and a restored
  account find every chat again. Integrity keeps the two networks apart: chat entries only in a
  chat, `Inbox` links only in the inbox, each under 900 bytes.
- **Read marks are private entries** (`ReadMark { at }`): they never leave their author's chain.
  `mark_read` writes one only when something from someone else is unread; the unread count is
  the other people's messages after the newest mark.
- **Messages** use the project chat's rules and shape (ADR 0038): 1–4,000 characters, replies
  cite a message, only the author deletes, no edits yet. The UI polls: the list every 8 s, an
  open chat every 5 s (web's `INBOX_REFRESH_MS` and `THREAD_POLL_MS`).
- **Starting a direct chat with someone you already talk to** posts into that chat instead
  (web's `start_direct_conversation`). Two people starting one at the same moment can end up with
  two chats; both work.

## Consequences
- Privacy rests on the seed staying secret. The membrane is not enforced at join time yet: anyone
  who learned a seed could join the network and read what it gossips, though validation stops
  them writing. A membrane proof (the invitation) checked at genesis would also get such peers
  warranted and blocked; that is a later integrity change.
- Who invited whom is visible: inbox links sit on the recipient's key and are authored by the
  sender. Only the chat itself stays hidden. Delivering invitations by remote call to online
  peers could hide even that; the link is kept because it also works when the recipient is
  offline.
- People hosted on a gateway keep their keys on that node (ADR 0010), so its operator could read
  their chats, as web's operators can today. Devices running their own conductor don't have that
  exposure.
- Not yet ported from web: renaming a group, adding or removing members, leaving, pins, mutes,
  edits, attachments, calls and the Public tab (project chats in the Messages list). Removing a
  member needs a rule for messages written after the removal; it comes with that screen.
- Adding the messages zome changes the conversation DNA hash and so the hApp's integrity version:
  hosted apps are reinstalled as in ADR 0028/0029 (people restore from the network). The
  directory and project DNAs are unchanged, so their networks carry on.
