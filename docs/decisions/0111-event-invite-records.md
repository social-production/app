# 0111. Event invitations recorded on the event, so organisers see who is invited

Status: **accepted** (2026-11-17). The owner chose "reset now" when asked (12:25 am
Melbourne), for the Detail Pages Cleanup's member list (design 2e v5,
https://claude.ai/artifact/UJEVvZ1b93rpoJoKMrWaMp §6: "every event, public or private, has
Members and Invited sides", with who invited each person and when).

## Context
An invitation is an `Inbox` link on the invitee's key in the conversation DNA, its tag the
event's seed encrypted from sender to invitee (ADR 0043, 0061). Only those two can open it,
so an organiser could list only the invites they sent, never a co-organiser's, and a public
event had no invites at all. An Invited list needs a record every organiser can read.

## Decision
1. **`Invite { agent, grant }`**, a new project-DNA entry, linked `Invites` from the event's
   space. The inviter writes it right after sending the inbox invite
   (`sendEventInvite` → `record_invite`); the seed still goes only to the invitee. Its author
   and timestamp say who invited and when.
2. **Integrity:** you can't invite yourself; in a private event the author must be an
   organiser (`grant` cites their `Organizer` record, `None` for the creator), as for
   `Organizer` grants. In a public event anyone may record an invite (sharing an event with
   someone is open there).
3. **Reads:** `event_invites` returns one row per invited person (their latest invite),
   leaving out invites a removed organiser wrote after their grant ended (ADR 0073). The UI
   shows on Invited those who haven't joined; joining moves them to Members. A declined
   invite is private to the invitee (ADR 0061), so it stays on Invited.
4. Records are never updated or deleted (as every project record).

## Consequences
- **Project network reset** (new entry and link types). The directory and conversation DNAs
  are unchanged: directory `uhC0kfM8SR1…` as on main.
- Invites sent before this reset aren't listed (they're on the old network anyway).
- Anyone holding a private seed could still pass it on (ADR 0061); such an invite isn't
  listed, since only organisers may record one.
