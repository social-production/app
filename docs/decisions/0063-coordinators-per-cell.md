# 0063. Update coordinators per cell, on startup, install and new clones

Status: **accepted** (2026-10-28). Corrects [0029](./0029-integrity-version-and-coordinator-updates.md)'s gateway step.

## Context
PR #62 changed the conversation DNA's coordinator (the inbox takes private-event seeds) but not
its integrity, so the DNA kept its hash. After the deploy, creating a private event live failed
with the old coordinator's "That isn't a chat.". Reproduced locally with a kept data dir: hApp v1
(old inbox), a sign-in, then v2 (new inbox, same conversation hash, new hApp version): the
person's v2 app, installed after startup, ran the old coordinator.

Holochain 0.7 keeps a ribosome per **cell** (`RibosomeStore` is keyed by `CellId`), and
`UpdateCoordinators` replaces only the given cell's. The gateway updated one cell per DNA at
startup ("one call covers every agent's cell"), and cells installed or cloned later could start
from the coordinator stored for that DNA hash.

## Decision
- At startup the gateway updates **every** cell of every app of the current hApp version.
- It also updates a person's cells right after `ensure_hosted` (signup, login, restore), and a
  person's new clone cells when their session is extended to them (`grant_new_cells`).

## Consequences
- More admin calls at startup and sign-in (one per cell); cheap next to genesis and joins.
- Verified locally: with the fix the same v2 data passes `tests/e2e/private-events.mjs`.
- A coordinator-only change now reaches every hosted cell on the next deploy.
