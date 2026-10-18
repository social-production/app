# 0022. The 24-word phrase is the agent seed

Status: accepted (2026-10-18)

## Context
Hosted users must be able to move to any node or app with their identity intact.

## Decision
The recovery phrase is 24 BIP-39 words encoding exactly 32 bytes of entropy, used directly as the Holochain seed (no BIP-39 password stretching). The agent key is that seed's ed25519 key, imported into lair the same way runtime-tauri does.

## Consequences
Same phrase, same agent key everywhere. Anyone holding the words can act as the person, which the signup screen says plainly.
