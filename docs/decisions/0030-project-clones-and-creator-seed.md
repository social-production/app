# 0030. Each project is a clone; its network seed names the creator

Status: accepted (2026-10-19). See docs/plans/M2-projects.md.

## Context
Each project needs its own network (ADR 0005), so its members and archivists hold its data. Rules
such as "only the creator writes the charter until M3" need every validator to agree on who the
creator is. Clone DNAs differ by their modifiers (network seed, properties). The browser
(`@holochain/client`), the gateway (Rust) and validators must derive the same DNA hash from them.

## Decision
- A project is a clone of the `project` role with network seed `<creator agent key>.<random nonce>`
  and no extra properties. `project_integrity::creator_in_seed` reads the creator from it.
- The directory's `ProjectListing` carries the seed. Validation requires the seed to name the
  listing's author, so nobody can list a project under someone else's name.
- Membership and demand/opposition signals are links whose target is their author (`Members`,
  `Signals` with a one-byte stance tag). There are no entries, and there is one live signal per agent
  (the coordinator replaces the old one).

## Consequences
- One plain string defines the network, so there is no risk of properties encoding differently
  between msgpack (JS) and YAML (Rust).
- A project's creator is fixed for the life of its network. Handing it over (M3 governance) is a
  proposal outcome, not a change of creator.
- Counting members or signals needs only link reads, without fetching entries.
