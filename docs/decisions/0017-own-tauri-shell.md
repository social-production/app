# 0017. Own Tauri 2 shell embedding the holochain crate

Status: superseded by [0021](./0021-holochain-runtime-tauri.md) (2026-10-18)

## Context
tauri-plugin-holochain is source-available (no open licence) and supports only up to Holochain 0.6. kangaroo-electron is desktop-only.

## Decision
native/ is our own small open-source Tauri 2 app that starts the conductor in-process (the holochain crate is CAL-1.0). Same code for desktop now and Android/iOS later.

## Consequences
We own a few hundred lines of runtime glue, and control versions and licence. The app also embeds `services/gateway` on localhost, so the UI has a single connection path (gateway → `/session` → `/ws`) for browsers and apps alike.
