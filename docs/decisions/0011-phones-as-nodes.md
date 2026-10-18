# 0011. Phones become light nodes

Status: accepted (2026-10-18)

## Context
If the cloud node is shut down, phone users must still be able to collaborate.

## Decision
Zomes are written to run unchanged on light nodes. Order: web + PWA + desktop first, then Tauri Android/iOS apps embedding a conductor (iOS via the wasmer-wasmi interpreter). M0 checks both platforms early.

## Consequences
Mobile constraints (background suspension, storage) shape the zomes from day one. Always-on peers relay between phones.
