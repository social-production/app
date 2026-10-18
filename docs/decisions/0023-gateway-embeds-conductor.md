# 0023. The seed node is one process: the gateway embeds the conductor

Status: accepted (2026-10-18)

## Context
The gateway must import seeds into the keystore and make admin calls. A separate conductor process would need an admin websocket and a separate lair server.

## Decision
services/gateway starts lair and the conductor in-process (as runtime-tauri does) and makes admin calls directly. Only the app interface listens, on localhost, behind `/ws`.

## Consequences
No admin port exists. One binary and one Dockerfile. The holochain binary is no longer downloaded for the seed node.
