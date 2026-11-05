# 0073. "Remove organizer": a revoke-tagged link, replayed by every reader (coordinator only)

Status: **accepted** (2026-11-05). Follows [0061](./0061-private-events-and-organizers.md)
(organizers are `Organizer` grant records; "revoking waits for its screen … it needs a rule for
what a revoked organizer wrote afterwards"). Parity item "Events, later parts: Remove
organizer" (`docs/plans/PARITY.md`).

## Context
web's `EventMembersPanel` shows "Remove organizer" (danger tone) on every organizer row but the
creator's, to anyone who can manage organizers. web-backend's `revoke_event_editor`: any
organizer may revoke any co-organizer; the creator's authority can't be revoked ("Cannot revoke
the event creator's organizer authority"); the person stays a member. Promoting again works.

In the app a co-organizer cites their `Organizer` record in what they write, and integrity
checks that record with `must_get_valid_record`, which can't see a later revocation
(validation must be deterministic). Deleting the `Organizers` link is allowed only to its
author, so another organizer (or the creator) couldn't remove a grant someone else wrote. A
new entry type would change the project DNA's hash and reset every project network.

## Decision
1. **A removal is an `Organizers` link tagged `revoke`** from the space to the grant it ends,
   written by any organizer (`project::remove_organizer(agent)`). Integrity already accepts any
   `Organizers` link, so the project DNA keeps its hash: no network reset.
2. **Every reader replays the ledger** (`organizers::ledger`): grants and removals in link-time
   order; each counts only if its author was the creator or held a grant in force at that
   moment. So a grant written by a removed organizer after their removal never counts, the
   grants they wrote before stay (as on web, where grants are independent rows), and a stray
   `Organizers` link from a non-organizer does nothing. One grant per agent is in force;
   promoting a removed person again writes a fresh `Organizer` record.
3. **What a removed organizer wrote afterwards is left out by readers**: in an
   organizer-controlled event, governance's `passed` ignores a vote-less pass whose proposal
   cites a grant that had ended before it was written, and the activities list leaves out an
   activity citing one (`organizer_ends`, `calls::lapsed`). Honest coordinators already refuse
   them (`role_of` finds no grant); this covers a modified coordinator, as ADR 0071's
   replayed moderation does.
4. **UI from web:** the members sheet's "Remove organizer" on co-organizer rows for organizers;
   the person moves back under Members (with "Promote to organizer"). A removal shows at once
   on the remover's device (network reads lag their own writes).

## Consequences
- Integrity still accepts records citing an ended grant; only readers drop them. Validation
  can't do better without a deterministic revocation (e.g. a grant with an expiry, or a
  membrane proof), which is an integrity change to batch with the next reset.
- Link times are the author's clock, as everywhere else in the replayed rules.
- Other reads that cite organizers (a single activity fetched by id, the rail) aren't
  filtered; they reach only what the list already left out through links to it.
