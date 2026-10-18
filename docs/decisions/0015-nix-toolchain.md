# 0015. Nix flake (holonix main-0.7) for the toolchain

Status: accepted (2026-10-18)

## Context
Holochain needs exact matching versions of holochain, hc, lair and the Rust wasm target.

## Decision
flake.nix pins holonix main-0.7. Local dev, CI and docs all use `nix develop`.

## Consequences
Reproducible builds. Contributors install Nix once.
