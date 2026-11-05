# 0075. Reports in direct and group chats, and invite declines as private inbox entries

Status: **accepted** (2026-11-05). Follows [0071](./0071-reports-and-moderation.md) (reports
replayed from votes), [0043](./0043-direct-messages.md) and [0045](./0045-group-options-edits-pins.md)
(chats cite invitations), and finishes [0061](./0061-private-events-and-organizers.md)'s "declines move
into a private entry". Parity item "Moderation, rest: reports in direct and group chats"
(`docs/plans/PARITY.md`).

## Context
web's Messages page uses the same `LiveChatPanel` for direct and group chats as for project
chats, with `reportTargetType="message"`: the bubble menu's Report, then "Vote to remove" /
"Vote to keep" while the report is open. web-backend's moderation for a `message` target:
- **Who votes:** the chat's members, never the message's author (`viewer_can_vote_on_target`);
  no standing check. The reporter's yes comes with the report.
- **Audience:** the chat's members other than the author, `max(1, members - 1)`, with no
  floor of 3 (`electorate_size_for_target`). Popularity is 0 (`load_target_engagement`).
- **1:1 path** (`is_tiny_private_dm`, audience ≤ 1): quorum 1 and the report may land on
  `removed` in one step, so in a direct chat the report itself removes the message.
- Otherwise web's quorums and shares as for any report (ADR 0071). In a group of three
  (audience 2) deletion is out of reach of the 3-vote floor: the report reviews, then the next
  vote dismisses it. Kept as web has it.

## Decision
- **Records** (conversation integrity, `report.rs`): `Report { message, reason, description,
  invite }` and `ReportVote { report, yes, invite }`, each listed under the chat's messages
  anchor (`Reports`, `ReportVotes` links, by the entry's author, as every chat entry). Both cite
  the writer's invitation, so a validator checks membership from the cited record (as messages
  do); a report is never about your own message, a vote never on a report about your own
  message. Nothing is deleted; a later vote replaces yours on replay.
- **Replay** (`sp_moderation::replay_chat`, `Thresholds::chat`): web's numbers above, with
  `tiny` (audience ≤ 1) for the 1:1 path in `advance` and `step`. Only votes and reports by
  people in the chat at the time count (the roster, ADR 0045).
- **Coordinator** (`reports.rs`): `report_message`, `vote_report`, `message_moderation(ids)`,
  returning the project DNA's `ModerationView` shape, so the UI reuses its mapping.
- **UI:** `ChatModeration` moved to `components/chat` with injected calls; `ConversationView`
  reads `services/chatReports.ts` and passes `reports` to `ChatPanel` (web's bubble menu,
  composer, "Removed for …" and the serious-harm reveal, already there from ADR 0071 part b).
- **Declines** (batched here, ADR 0061): a private `Declined { seed }` entry, valid only in
  the inbox (`decline_invite`, `declined_invites`). The Invites page reads it instead of
  `localStorage`, so a declined invite stays hidden wherever your inbox cell runs (every
  browser on the same gateway account), not just in one browser.

## Consequences
- **Integrity change in the conversation DNA only:** merging resets every direct and group
  chat and the inbox (test data only). Project and directory hashes are unchanged (checked
  with `hc dna hash` against main's bundles).
- A declined invite still comes back after a restore on a new conductor (private entries
  aren't on the DHT); the same as read marks and list settings.
- web keeps a hidden message's photos visible behind the reveal; the app hides them until
  revealed (as in ADR 0071 part b).
