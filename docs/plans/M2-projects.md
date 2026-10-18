# M2 — Projects (plan, decided 2026-10-18)

Three steps, each tested and deployed on its own.

## Decisions
| Topic | Decision |
|---|---|
| UI | Start of M2: port `web/`'s design system + app shell and redo the M1 screens with it; afterwards every milestone ports its own screens from `web/`. No new UI design. |
| Who creates | Anyone. **New** members: at most 2 projects per 7 days (validated from the creator's own chain); Full members unlimited (the listing cites their standing like a vouch does). |
| Project network | Each project is a clone of the `project` DNA with its own network seed, so members (and archivists) hold its data. |
| Archivist | Seed nodes run an archivist agent that joins **every** project clone, so projects stay available when members are offline. Community nodes can run the same thing. |
| Data | The charter keeps web's project fields as they are: title, description, mode (`productive` / `collective-service` / `personal-service`), subtype (`standard` / `software` / `asset-management`), tags, location label, phase/stage label. Until M3, the creator edits the charter; from M3, changes go through proposals. |
| Feeds | Project feed with web's sorts (`trending`, `recent`, `top`, `oldest`) and windows (`today` / `week` / `month` / `all`), plus text search, served by the node's local index (`sp_index`). The trending formula is ported from `web-backend/app/services/feeds/ranking.py`. |
| Signals | Demand/opposition signals per project (one live signal per agent), needed for trending. |

## M2a — real UI shell (no backend change)
- Port `web/src/app.css` and the design tokens. The shell (top bar, left rail, right rail placeholder, mobile
  bottom nav, page header, toasts, sheets) is rebuilt in Svelte 5, split into ≤300-line components.
  The visual output stays identical to `web/`.
- Redo Profile, People and the account screens (signed-out intro, phrase sign-in) with web's components.
- Check: side-by-side screenshots of `web` (live, read-only) vs `app/ui` at 1280 and 390 px, plus the browser tests.

## M2b — projects backend
- directory: `ProjectListing { network_seed, summary, creator_standing }`, links for the all-projects anchor, tags and creator.
  The rate limit for New creators lives in integrity.
- project DNA zomes: `Charter` (create/update by creator in M2), `Membership` (join/leave), `Signal` (demand/opposition).
- Gateway:
  - `/session/cells` re-grants the device key on new clone cells (needed after join/create).
  - Archivist agent: polls listings and joins new projects.
  - `sp_index`: trending/recent/top/oldest, windows, search; served at `/index/projects`.
- Tests: sp_index ranking units (ported formula), scenario tests (create, join, leave, signal, New-member limit, forged standing rejected),
  and the archivist joining a project created on another node.

## M2c — project screens from `web/`
Public project feed with filters, project detail (header, about, members, signals, join/leave), the
create-project flow, and editing the charter (creator).
