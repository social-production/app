# 0019. SvelteKit 3 with #lib imports

Status: accepted (2026-10-18)

## Context
A fresh app should not start on a framework version that is about to need migrating.

## Decision
SvelteKit 3.0.x, configured through the sveltekit() Vite plugin (no svelte.config.js), with #lib/... subpath imports that use explicit extensions.

## Consequences
Imports read '#lib/holo/index.ts'. Boundary checks match '#lib/'.
