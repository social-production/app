# 0031. Seed nodes archive every project and serve the project feed

Status: accepted (2026-10-19). Builds on [0012](./0012-local-index.md).

## Context
A project's data lives only with the agents in its clone network. When its members are offline it
would disappear. The feed (web's trending/recent/top/oldest sorts with windows and search) needs
counts from every project, and a browser can't join hundreds of networks just to rank them.

## Decision
- The gateway runs an **archivist** agent (`archivist-<version>`, `node/archivist.rs`). Every 15 s it
  reads `list_projects` from the directory and clones each new project's seed. It holds the data
  (full arc) but never joins as a member.
- In the same pass it reads each project's `get_project` and rebuilds an in-memory
  `ProjectIndex` of cards. `GET /index/projects?sort&window&q&offset&limit&tzOffset` serves it,
  ranked by `sp_index` (the trending formula ported from web-backend).
- The index is non-authoritative (ADR 0012). Opening a project always reads its own network.

## Consequences
- Projects stay readable with every member offline. Community nodes can run the same archivist.
- The feed lags writes by at most one archive round, plus gossip.
- The index is rebuilt in full each round. That is fine for hundreds of projects. Before thousands, it moves to
  incremental updates and gets persisted.
