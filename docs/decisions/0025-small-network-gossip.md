# 0025. Gossip tuned for a small network

Status: accepted (2026-10-18)

## Context
M0 measured >4 min first contact over the relay: Kitsune2's defaults wait 5 min before retrying a peer and time out rounds after 15 s.

## Decision
All nodes merge `sp_core::network::GOSSIP_TUNING_JSON` (30 s initiate, 60 s minimum per peer, 60 s round timeout) into `network.advanced`.

## Consequences
Faster first contact at slightly more background traffic. Revisit when the network is large.
