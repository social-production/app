# 0069. Events for a private community: a private event listed on the community by its seed

Status: **accepted** (2026-10-31, cloud session). Builds on [0061](./0061-private-events-and-organizers.md)
(private events are clones with secret seeds) and [0068](./0068-channels-and-communities.md)
part (d) (private communities are clones too). **Resets the project network** (integrity
change): test projects, events, help requests, channels and communities start again. The
directory and conversation DNAs keep their hashes.

## Context
web's third event audience, `private_community` (web `CreateEventVisibilityPanel`,
`CreateEventPage`; web-backend `services/events/helpers.py::create_event`,
`events/actions.py::join_event`, `feeds/selects.py`):
- The Audience step offers Public, **Private community** ("Only members of one private
  community can find it on feeds and the map.") and Invite only. The Who step then asks for
  exactly one private community ("Pick exactly one private community. Its members can discover
  this event.", suggestions labelled "<name> (Private)") plus optional organizers; Control and
  Plan work as for invite-only events.
- web stores the community as `home_community_id` and also tags the event with it. Members of
  the community see the event in the community's feed and their own feeds and join it without
  an invite; others get "Join the event's community to join this event".

The app can't put the event inside the community's network: a private clone holds exactly one
space, and only the creator its seed names may found charters there (ADR 0065). Nothing in the
community's network could carry another network's address either.

## Decision
1. **The event is a private event** (ADR 0061): its own clone, `.private.` or `.organized.`
   seed, same rules, organizers and up-front plan. Its charter is tagged `@<community slug>`,
   as web tags it; the UI reads "private and tagged with a community" as `private_community`
   (`domain/projects.ts` `audienceOf`). The DNA itself still only knows private or not.
2. **A new link in the project DNA, `ScopeItems`:** community space → the lister, tag = the
   event's seed. Valid only in a private clone, on a community's charter, for a private seed
   (`Audience::of_seed`) whose creator is the link's author: only an event's creator lists it.
   The coordinator's `add_scope_item(seed)` writes it once; `scope_items()` lists them, newest
   first. Deleting stays the author's (the general link rule).
3. **The community's page is its feed:** members see the listed events as web's event cards,
   read from each event's own network (`services/closedCommunities.ts` `communityEventCards`);
   non-members (an invite link before joining) see none. A member opens a card and joins as on
   a public event; the creator is still invited through their own inbox for their other
   devices.

## Deviations
- **Seed holders can join:** web refuses non-members, but here anyone who has the event's seed
  can join, as with invite-only events (ADR 0061). Only members can read the seed from the
  community (and removed members still can, ADR 0068 "Removal doesn't lock someone out").
- **Not in members' Home feed yet:** the app's Home scope doesn't show joined channels' and
  communities' items at all yet; private-community events join it when that lands. Region and
  the map wait on places (M6).
- **Listing is the creator's:** web's event belongs to the community from creation; here the
  creator writes the listing right after creating the event. If that write fails, the event
  exists but isn't listed (the creator can still share its address).

## Consequences
- One more project-network reset on the test network.
- Each listed event a member opens costs a clone on their node (as invites do); gateways keep
  idle clones cold (ADR 0064).
