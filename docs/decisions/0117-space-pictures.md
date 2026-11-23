# 0117 Projects, services, proposals and events have a picture and cover

Status: accepted (owner, 2026-11-24: "reset today, reseed, but thread and post dont need
picture and cover of course, requests dont either")

## Context
ADR 0109 gave channels and communities a picture and cover (directory `ScopeImage` records named
by hash from the charter). The owner wants every project-like space to have them too, from the
moment it is made, and on its page.

## Decision
- Project integrity's `charter::takes_art(mode, title)`: channels and communities (not the
  Platform), and every other mode except help requests, threads and posts. A charter and an
  `Edit` proposal may name a picture or cover only when it holds.
- Create proposal and Create event start with the cover and picture (optional: the type's tile
  stays). Propose edit shows them with camera buttons; a new one is voted on with the rest.
- The page shows the cover with the picture over its corner; feed rows use the picture as
  their tile. The seed node's cards already read both from the charter.
- The directory listing still names art only for channels and communities, so the directory
  network is unchanged.

Project integrity change: a **project network reset** (directory and conversation unchanged:
accounts, follows and chats stay). Demo content is re-run afterwards.
