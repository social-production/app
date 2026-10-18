# 0004. Fresh SvelteKit UI carrying the current design

Status: accepted (2026-10-18)

## Context
web/ is ~106k lines in legacy Svelte mode with an adapter shaped around centralized semantics.

## Decision
Build a new SvelteKit app (Svelte 5 runes, TS strict, static SPA, PWA). Port visual components and design tokens from web/src (tokens.css came from web/src/app.css), but not the data layer.

## Consequences
More work up front, but clean layering from day one. Testers keep a familiar look.
