# 0013. Clients ship a list of nodes and fail over

Status: accepted (2026-10-18)

## Context
Railway must be node #1, not the only node.

## Decision
ui/src/lib/holo/nodes.ts tries the local conductor, then configured gateways in order, and later a list announced in the directory. Bootstrap/relay URLs are lists too.

## Consequences
Losing one host is invisible to users with another reachable node.
