# Domain model

The planned entry/link types per DNA, and where each of the 111 FastAPI tables ends up.
The legacy map is **reference only**: no legacy data is imported (ADR 0001). It exists to
make sure parity misses nothing. Types are filled in milestone by milestone.

## directory (global)

| Entry / link | Milestone | Notes |
|---|---|---|
| `Profile` | M1 | name, bio, avatar ref. Latest update wins |
| `Follow` link | M1 | agent → agent |
| `Vouch`, `StandingGrant` | M1 | see GOVERNANCE.md#standing |
| `ProjectListing` | M2 ✅ | the project's network seed `<creator>.<nonce>` (ADR 0030) + title, summary, mode, tags, location, phase; the creator's Full-standing proof, or the New-member limit (2 per 7 days) |
| `Place` | M6 | locations for map/discovery |
| `NodeAnnouncement` | M1 | community seed/gateway URLs for failover |
| `Feedback` | M5 | platform feedback |
| platform-level `Proposal/Vote/Outcome` | M3/M6 | bot reviews, board membership, platform channel |

## project (clone per project)

| Entry / link | Milestone | Notes |
|---|---|---|
| `Charter` | M2 ✅ | title, description, mode/subtype, location, phase, tags. The creator edits it until M3; then only Outcomes |
| `Members` link | M2 ✅ | "members" anchor → the member's own key; leaving deletes the link (ADR 0030) |
| `Signals` link | M2 ✅ | "signals" anchor → the agent's key, tag = demand or opposition; one live per agent |
| `Proposal`, `Vote`, `Outcome`, `Challenge` | M3 | generic governance |
| `Thread`, `Post`, `Comment`, `Reaction` | M4 | discussion |
| `ChatMessage` | M4a ✅ | project chat, `discussion` zome: body + optional `reply_to`; anyone may post, only the author deletes, never edited (ADR 0038) |
| `Activity`, `Role`, `Assignment`, `Rating` | M5 | activities + scheduling |
| `ServiceRequest`, `AvailabilityRule`, `SlotHold` | M5 | service projects |
| `ExecutionReceipt` | M6 | board actions citing Outcomes |

## conversation (clone per chat)

| Entry | Milestone | Notes |
|---|---|---|
| `Message`, `Attachment` ref | M4 | membrane admits listed participants only |
| read markers, pins | M4 | **private** entries (never published) |

## Not on the DHT

- **Notifications** are derived locally from signals and new links. Nothing is stored centrally.
- **Search / feeds** are a local index (`sp_index`) on each full node (ADR 0012).
- **Settings** are private entries or device-local storage.

## Legacy table → P2P map

| Legacy tables | Becomes |
|---|---|
| `users`, `user_settings`, `user_follows` | `Profile`, private settings, `Follow` |
| `account_stances`, `meaningful_actions`, `bot_reviews`, `bot_review_votes`, `bot_mark_cooldowns`, `board_standing_votes` | `Vouch`, `StandingGrant`, `BotReview` kind |
| `projects`, `project_tags`, `project_signals`, `project_values`, `project_value_importance_votes` | `ProjectListing` + `Charter` (+ value votes as a special kind) |
| `project_memberships`, `event_memberships`, `event_editors` | `Membership` (roles via Outcomes) |
| every `*_request` + `*_request_votes` / `*_change_votes` (edit, phase, update, link, PR, merge capability, repository replacement, service settings) | `Proposal` kinds + `Vote` |
| `project_plans`, `*_plan_votes`, `*_plan_value_votes`, `*_plan_criterion_ratings` (+ event_) | `PlanSelection` special kind |
| `project_activities`, `*_activity_roles`, `*_assignments`, `*_ratings`, `*_history_completions` (+ event_) | `Activity`, `Role`, `Assignment`, `Rating` |
| `project_service_*` | `ServiceRequest`, `AvailabilityRule`, `SlotHold` |
| `project_updates`, `project_links`, `detail_links`, `project_conversions`, `project_inherited_decisions`, `project_revert_history`, `governance_decision_history` | Outcomes and their apply rules; history = the Outcome chain |
| `posts`, `post_links`, `threads`, `thread_tags`, `comments`, `comment_attachments`, `content_votes` | discussion entries + `Reaction` |
| `reports`, `report_votes` | `Moderation` proposal kind |
| `platform_board_memberships` | `BoardMembership` kind + `ExecutionReceipt` |
| `conversations`, `conversation_members`, `messages`, `message_attachments`, `conversation_pins`, `subject_chat_reads` | conversation DNA |
| `blobs` | chat files: `Chunk` entries in the conversation DNA (ADR 0049); others open |
| `notifications` | local derivation |
| `searchable_documents` | `sp_index` |
| `locations` | `Place` |
| `platform_feedback` | `Feedback` |
| `channels`, `communities`, `scope_*` | open item (below) |
| `scope_invites`, `pending_invites` | dropped: the membrane is open. Optional share links only |
| `help_requests`, `help_request_*`, `events`, `event_*` (non-plan) | open item (below) |

## Open placement questions (decide at the milestone)

1. **Events and help requests**: their own DNA clones (same lifecycle machinery as projects,
   with `properties.kind`), or entries inside a project clone? The leaning is clones of the project DNA with a kind.
2. **Communities and channels (scopes)**: clones of the project DNA with `kind: community`, or a separate scope DNA?
3. **Files (`blobs`)**: chunked entries (Holochain entries max ~4 MB) vs a content-addressed blob zome with
   member-only replication. **Chats: decided** ([ADR 0049](decisions/0049-chat-attachments.md)): 1 MB
   `Chunk` entries in the chat's own DNA, named in order by the message. Other files (avatars, project
   photos) are still open.
