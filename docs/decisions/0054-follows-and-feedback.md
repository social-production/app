# 0054. Follows and platform feedback in the directory (M5a)

Status: **proposed** (2026-10-25). Not built yet; the next session builds it from this.
Plan: [`docs/plans/M5-work.md`](../plans/M5-work.md) M5a.

## Context
web lets people follow each other (web-backend `services/users.py`): a follow is `accepted`
at once, or `pending` when the target has turned on "require follow approval", until the
target accepts or rejects it. Follows drive the profile counts and people sheets, the
personal feed and Home scope, `follow` notifications (`new-follower`, `follow-request`,
`follow-accepted`), mention ranking (ADR 0047), and two privacy switches (hide the personal
feed / profile activity from non-followers). web's Feedback page (left rail) lists platform
bugs and suggestions with up/down votes, filters (All / Bugs / Suggestions) and sorts.
The app has neither. DOMAIN_MODEL planned a `Follow` link in the directory (M1) and
`Feedback` there too (M5).

## Decision (proposed)
- **Directory integrity change, one reset:** both land together so the directory network
  resets once (the owner accepts resets until parity, STATUS "Owner direction").
- **Follows as links, written by the follower:** `Follows` (follower → followed) and its
  mirror `Followers` (followed → follower), both made by the follower; unfollow deletes both.
  Validation: author = follower, no self-follow.
- **Approval without a server:** a public `FollowPolicy { require_approval }` entry, latest
  wins (part of the profile settings). When it is on, a follow counts once the target has made
  an `Accepts` link (followed → follower); rejecting deletes nothing the follower owns, it
  just never accepts (web deletes the row; the follower's link here stays "pending" until
  they withdraw). Counts and lists read accepted follows only.
- **Privacy switches are display rules, not secrecy:** everything in the directory is public
  to its network, so "hide from non-followers" means the app doesn't show it to them, as on
  web's server. Say so in Settings' copy (like ADR 0043 does for chats).
- **Feedback:** a `Feedback { kind: bug | suggestion, title, body }` entry linked from a
  "feedback" anchor, and `FeedbackVote` links (tag up/down, latest per voter wins), as
  project signals do (ADR 0030). Sorting via `sp_index` (trending / recent).
- **Notifications** stay derived on the device (ADR 0046): `new-follower` / `follow-request`
  from links to you, `follow-accepted` from `Accepts` links to you; the Settings switch
  "Follows" joins the ones from PR #49.

## Consequences
- Everyone on the test network signs up again after the deploy; projects and chats tied to
  old agents become unreachable (as after earlier directory changes).
- Unblocks the PARITY.md items listed under M5a in the plan.
- Open: whether the gateway's people index (`node/people.rs`) should carry follower counts for
  guests, or guests read them through `/public/read` (add the follow reads to its allowlist).
