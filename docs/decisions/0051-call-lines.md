# 0051. Call lines in direct chats: a public `Call` entry, the first report per call shows

Status: accepted (2026-10-24). Builds on [0050](./0050-voice-calls-direct-signals.md) and
[0043](./0043-direct-messages.md).

## Context
web leaves a centred line in a direct chat after every voice call (`LiveChatPanel`
`callLineText`): "Voice call · 4:12" for a call that connected, "Call declined", and for an
unanswered call "No answer" to the caller and "Missed call" to the person called. Its
`session.ts` `logFinishedCall` reports the call when it ends: both phones report a call that
connected, the caller reports one that didn't (declined, no answer, not available, busy), and
nothing is logged when the caller's microphone failed. web-backend's `log_call` stores a
`kind: "call"` message with `call_id`, `call_outcome` and `call_duration_seconds` (clamped to a
day), keeps the first report per `call_id`, and shows it with an empty preview in the list.
ADR 0050 shipped calls without these lines because the conversation DNA had no place for them.

## Decision
- **Integrity change (conversation DNA only):** a public `Call` entry (`call_id`, `outcome`,
  `seconds`, the author's `invite`) under a new `Calls` link on the chat's messages anchor.
  Validation: direct chats only, written by someone in the chat, web's three outcomes, at most
  a day, and only a completed call has a length. The rules live in a new `crates/sp_call`
  (not `sp_core`, so the directory and project DNA hashes stay the same).
- **"The first wins" without a server:** `log_call` is a no-op when this node already sees a
  line for that `call_id`; when both phones write at once, `list_messages` shows only the
  oldest per `call_id` (`sp_call::first_per_call`), so everyone sees one line.
- Call rows come back from `list_messages` as `kind: "call"` with `call: { outcome, seconds }`,
  by whoever logged them, so they count as unread for the other person (as on web) and are the
  chat's last line with an empty preview (web-backend's `_message_preview`).
- **UI ported from web:** `callLineText` (`components/chat/chatFormat.ts`), the line
  (`ChatNote.svelte`, web's markup and the already ported `.call-line` styles), and
  `logFinishedCall` (`CallSession.logLine`), with web's same reporting rules. The app-only
  "Not available right now" (ADR 0050) logs a missed call, as web's `unavailable` does.

## Consequences
- The hApp version changes: hosted apps reinstall and the direct/group chats made before the
  deploy stay on the old conversation network (test chats only, as with ADR 0048 and 0049).
  Projects and people are untouched.
- A call line can't be edited, pinned, replied to or deleted (web's can't either).
- When the other person is busy, the app answers the invite with `reject`, so the caller's
  line reads "Call declined" where web would log a missed call.
- A call whose page closes before it ends (both phones gone) leaves no line, as on web.
