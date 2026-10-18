# 0012. Feeds and search are a local index on every full node

Status: accepted (2026-10-18)

## Context
There is no central SQL for ranking or full-text search.

## Decision
crates/sp_index builds a non-authoritative index from the DHT data a node holds. Thin clients query the node they are connected to. All nodes run the same ranking code.

## Consequences
No node is authoritative. Results can differ slightly while gossip catches up.
