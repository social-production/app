# 0061. Invite-only events: the seed says private or organizer-run; organizers are grant records (M5c part 2)

Status: **accepted** (2026-10-28). Builds on [0043](./0043-direct-and-group-chats.md) (secret
seeds, the inbox) and [0057](./0057-events-as-project-clones.md) (events are project clones).
**Resets the project network** (integrity change): everyone's test projects and events start
again. The directory and conversation DNAs keep their hashes.

## Context
web (web-backend `services/events/helpers.py::create_event`, `events/phases/gates.py`,
`events/actions.py`, `services/invites.py`; web `features/create/event/**`,
`features/invites/InvitesPage.svelte`):
- An event's **audience** is `public`, `private_community` or `invite_only`. Public events are
  always collaborative. Private events choose **governance**: `collaborative` (the public
  flow inside the private audience) or `organizer_controlled`.
- **Organizers** are the creator plus co-organizers (`event_editors`), chosen at creation or
  granted later by any organizer (granting also adds them as members). Only organizers invite.
- **Organizer-controlled** events skip signals, values and votes: the create flow takes the
  plan up front (title, description, schedule, stages) and the event starts in Activity with
  that plan approved and leading. Organizers' edits, updates and phase changes apply at once
  (`auto_execute`); only organizers propose them, create activities, and they can't return
  the event to Proposal. Members still join and take roles.
- **Invites** wait for an answer: the invitee gets an `evt-invite` notification ("You were
  invited to an event: <title>. Accept or decline it in Invites.") linking `/invites`, where
  they accept (join, open the event) or decline. The left rail shows "Invites" with a count
  while any wait. Leaving a private event warns that a new invite is needed to come back.

## Decision
1. **The network seed says the audience** (`sp_event::Audience`): `<creator>.<nonce>` is
   public, `<creator>.private.<nonce>` is invite-only and collaborative,
   `<creator>.organized.<nonce>` is invite-only and organizer-controlled. Validators read it
   from `dna_info`, so the rules below need no lookups and can't change after creation (as
   on web, where governance is fixed at creation). Private nonces are 24 random bytes, as for
   chats: the seed is the event's secret address. `creator_in_seed` is unchanged (the creator
   is still before the first dot).
2. **Private events are not listed in the directory.** Their address is the seed:
   `/events/<seed>` opens one (a listing id has no dot, a seed always does). Anyone who has
   the seed can read and join, as with chats (ADR 0043): privacy rests on the seed staying
   secret.
3. **Invites go through the inbox** (the conversation base cell, ADR 0043): the same
   encrypted `Inbox` link, carrying the event's seed. The messages coordinator accepts event
   seeds as well as chat seeds (coordinator only, so the conversation DNA keeps its hash);
   the UI splits them by shape. The creator sends one to themselves, so their other devices
   find the event. **Accept** joins (a `Members` link) and opens the event; **Decline** is
   kept on the device (like Public-tab read marks, ADR 0044); an invite whose event you
   joined or declined no longer counts.
4. **Organizers are `Organizer { agent, grant }` records** in private events only: the
   creator writes them with `grant: None`; any other organizer cites their own `Organizer`
   record as `grant`. `must_get_valid_record` makes the chain of grants valid transitively.
   Revoking waits for its screen (web's members panel); it needs a rule for what a revoked
   organizer wrote afterwards.
5. **Organizer-controlled rules (integrity, `.organized.` seeds):** every proposal except a
   link request mirrored from another project, and every activity, must be by an organizer
   (the creator, or citing their `Organizer` record in the new `organizer` field). An
   `Outcome` with `passed: true` and no votes is valid for those proposals. Values,
   importance votes and signals are refused, and a return can't go to Proposal. The
   coordinators close each organizer proposal at once, skip the signal gate and the
   recount, and take the newest approved plan as the leader.
6. **Creating an organizer-controlled event** writes the charter, the co-organizers'
   `Organizer` records, then three closed proposals: Proposal → Event Plan, the event plan
   (schedule required, as web's Plan step), Event Plan → Activity citing it. So the event
   opens in Activity with web's "Set when members approve" replaced by the plan's dates.
   web stores an empty demand note for that plan; plan integrity (`sp_core`) requires one,
   so the app writes "Set by the organizers; this event takes no support signals." rather
   than change `sp_core` (which would reset every network).
7. **UI ported from web:** the Audience step (Public, Invite only), the Who step (Invite
   people, Organizers (optional)), Control and Plan steps, the Overview rows, `/invites`
   with Accept/Decline, the left rail's Invites row and count, the `evt-invite`
   notification, `PrivateMark` on private events, and the leave warning. **Private
   community** waits for communities (M5d) and is left out of the Audience step until then.

## Consequences
- One more project-network reset on the test network.
- Seed holders can't be stopped from reading what the event gossips, and a non-organizer
  who has the seed can still pass it on (web's "only organizers invite" is a UI rule here).
  A membrane proof at join time is the later fix, shared with chats.
- Who invited whom is visible on the inbox network (ADR 0043); which event is not.
- Gateway-hosted people's events are readable by that node's operator, as on web.
- A declined invite comes back on another device until declines move into a private entry
  (a conversation integrity change, to batch with the next one).
- Private events don't appear in the feed yet (it is built from the directory's index);
  their members reach them from Invites, notifications, the rail and links. Merging the
  events open on this node into the feed comes next.
