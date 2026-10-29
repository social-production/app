# 0068. Channels and communities are spaces; tags name them by handle

Status: **proposed** (2026-10-29, cloud session). Builds on [0065](./0065-public-commons-network.md)
(public spaces in the commons, private ones as clones) and [0061](./0061-private-events-and-organizers.md)
(organizer grants). Plan: `docs/plans/M5-work.md` § M5d.

## Context
web's scopes (`features/communities`, `channels`, `shared-directory`, ~2,500 lines; web-backend
`services/scopes.py` + `scope_governance.py`, ~1,650 lines):
- **Channel:** a public topic, `/channels/<slug>`, any member may tag items with it; its creator
  is its first manager. Reserved slugs `platform`, `stewardship`.
- **Community:** `/communities/<slug>`, join policy `open` or `closed` (private: invite only,
  hidden from people outside it; items tagged *only* to it are hidden too); governance
  `collaborative` (members vote) or `organizer` (closed ones only; managers decide).
- **Handles:** slug = canonical form of the name, unique across channels and communities.
- **Membership:** join/leave (closed: by invite link with a token, or a direct invite the
  person accepts); roles `member` / `manager`.
- **Scope proposals** (7 days, 66% of votes cast, quorum by population): `edit` (name,
  description, image), `remove_member`, `to_organizer`, `to_collaborative` (managers vote).
- Items (projects, events, help requests, later posts/threads) carry channel/community tags;
  help requests and threads require at least one. Feeds per scope; the left rail lists the
  scopes you joined; Discover lists them all; Home merges your scopes' items (ADR 0055).

## Decision
1. **A public channel or community is a commons space** whose charter mode is `channel` or
   `community` (new crate `sp_scope`: kinds, handle rules, join policies, reserved slugs).
   The charter's title is the handle and its subtype the join policy (`open`; `closed` only in
   a clone). A **closed community is a clone with private events' seeds**:
   `<creator>.private.<nonce>` (members vote) or `<creator>.organized.<nonce>` (organizer-run),
   so web's "only closed communities can be organizer-run" holds by construction and ADR
   0061's `Organizer` grants are its managers unchanged. Membership is the space's `Members`
   link; a public scope's creator is its first manager (the space's creator).
2. **Handles are claimed, first come first served, from the directory's listings** (which
   already accept any kind slug, so the directory DNA doesn't change). Readers take the
   **earliest** listing of a slug (by its timestamp, ties by hash), so a later duplicate is
   ignored rather than refused (no global lock in a P2P network). Creating looks the slug up
   first and refuses a taken handle, so duplicates come only from simultaneous creation. The
   gateway index serves the scopes (`/index/scopes`) for Discover and tag search.
3. **Tags are handles.** A charter's `tags` already holds strings; a scope tag is
   `#<slug>` for a channel and `@<slug>` for a community (plain words stay free tags until the
   UI stops offering them). The gateway index files each card under its scope tags; per-scope
   feeds and Home read that. Items tagged only to closed communities are never listed by the
   gateway index (it can't read their seeds anyway; members see them through their clone).
4. **Scope governance reuses the project DNA's proposals**: `Edit` (description and image;
   the handle stays, since tags name it) and a new `RemoveMember { agent }`, counted by the
   existing tally (web: 66%). A passed removal hides the member's earlier `Members` link; in a
   closed community they would need a new seed to come back. Organizer-run communities
   decide at once (ADR 0061's path). web's `to_organizer` / `to_collaborative` aren't ported:
   the seed fixes the governance (a switch would mean moving to a new clone).
5. **Invite links** carry the clone's seed in the URL fragment (never sent to a server);
   direct invites reuse the inbox invite (ADR 0061's `send_invite`).

## Consequences
- **Integrity change in the project DNA only** (scope modes, `RemoveMember`): the project
  network resets once; accounts and chats keep theirs.
- Posts and threads (web's other tagged items) still don't exist in the app; scope pages show
  projects, events and help requests until they do.
- Parts, each ending deployed: (a) DNA + scenarios; (b) create channel/community flows,
  scope pages with feeds, join/leave, left rail, Discover; (c) tagging in create flows and tag
  chips with web's icons; (d) closed communities, invites, members sheet; (e) scope
  governance sheets (settings, votes).
