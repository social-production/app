# 0024. Pair devices instead of sharing a key

Status: accepted (2026-10-18)

## Context
One agent key running on two conductors at once forks its source chain.

## Decision
Every app install has its own agent. People join devices to one person with mutual `DeviceLink` entries (both halves required). Importing the phrase into the native app is reserved for moving off a hosted node (M7), not for running both at once.

## Consequences
Votes are later counted per primary agent (M3). The UI shows one person with several devices.
