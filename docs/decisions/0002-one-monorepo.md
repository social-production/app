# 0002. One monorepo: social-production/app

Status: accepted (2026-10-18)

## Context
Changing an integrity zome changes the DNA hash, so the UI, zomes and native wrappers must release in lockstep. The old split into web/ and web-backend/ needed manual coordination.

## Decision
One repo (github.com/social-production/app) holds the DNAs, pure crates, gateway, UI, Tauri app, deploy and tests.

## Consequences
One CI run checks everything together. App-store wrappers point at ui/build, so they need no repo of their own.
