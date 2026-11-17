# 0109. Pictures for channels and communities, and the Platform's open votes

Status: **accepted** (2026-11-17). The owner approved the Channel Pictures mockups
(https://claude.ai/artifact/3t4YWuh5PsGw2AorUfrY8f, design 2c) and, asked on 17 Nov, chose the
directory reset so the pictures show everywhere ("Directory reset"). This is the rest of ADR
0095 as well (#platform voters), in the same reset. The parity session planned this ADR; the
owner handed it to the UX session that day.

## Context
- A channel or community shows a `#` or people glyph on a pale band. The mockups give each a
  square picture and a 3:1 cover, shown on its page, in feed rows tagged with it, the left
  rail, Discover and Search. The creator may add them when creating, with no vote. Later
  changes follow the scope's own rules: a member vote (66%, ADR 0033) in public and
  collaborative scopes, organizers at once in organizer-run ones (ADR 0068 part d).
- A scope's settings live in its own clone of the project DNA: its `Charter`, changed by
  passed `Edit` proposals. Feed rows, the rail, Discover and Search read only the directory
  `ProjectListing`, never the clone.
- ADR 0095 left two things for later: any Full member may vote on a `#platform` project
  (web's `is_platform_tagged`), and its population is the weekly active count.

## Decision
1. **Images are directory records.** `ScopeImage { mime, bytes }` (JPEG, PNG or WebP, at most
   512 KB, resized in the browser as covers are), content-addressed, so the same image is one
   record. Anyone may write one; nothing points to it until a charter or a passed edit does.
   `scope_image(hash)` reads it.
2. **The charter names them.** `Charter` gains `picture` and `cover` (`Option<EntryHash>`);
   only a channel or community may set them, at creation. `ProposalKind::Edit` gains `picture`
   and `cover` (`Option<ImageChange>`: `Set(hash)` or `Remove`), again only for scopes. The
   current picture is the charter's, replaced by each passed edit in order, as the
   description already is. The Platform keeps its globe (it takes no proposals, ADR 0095).
3. **Rows show the current ones.** `ProjectListing` gains `picture` and `cover` as created. The
   seed's index reads each public scope's current charter (it already does, for titles and
   summaries), so its cards (feed rows, the left rail, Discover, Search) carry the picture and
   cover with passed edits applied; the UI fetches the image by hash and caches it. A closed
   community is never indexed; its members read its clone.
4. **`#platform` votes are open.** In a project whose charter is tagged `platform`, a vote may
   come from someone who isn't a member: `Vote.membership` becomes `Option<ActionHash>`,
   `None` allowed only there. Integrity can't read standing in another DNA, so the recount
   (ADR 0040, `decide.rs`) keeps only voters with Full standing before the proposal, as it
   already does for members, and sizes the population from the directory's weekly active
   count at the proposal's time (ADR 0095), never smaller than the member snapshot. The
   Outcome check in integrity stays a member-snapshot tally; readers trust the recount.
5. Everything else stays: records are never updated or deleted (images too), the handle is
   fixed, a personal service can't carry `#platform`.

## Consequences
- **Directory and project network resets**, together (new entry types and fields). The
  conversation DNA is unchanged. Test data only; everyone signs up again.
- The listing keeps the picture it was created with (only its creator may update it); readers
  use the index's card or the clone's charter, both current.
- An image nobody points to costs storage on the holders of its DHT arc; covers have the same
  property (ADR 0107).
- The moderators' merge capability on #platform software projects (ADR 0095 "Powers") stays
  next; it needs no reset.
