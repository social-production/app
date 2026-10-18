# 0027. The recovery phrase is the sign-in; passkeys are optional

Status: accepted (2026-10-18)

## Context
M1 first made passkeys mandatory. On devices without a built-in passkey store (many Linux
desktops, older phones) browsers fell back to asking for a USB "security key", which made no
sense to people and blocked sign-in.

## Decision
Signing up, signing in and restoring all work with the 24 words alone (`/account/open`). A
signed-in device may optionally save a passkey (`/account/passkey/*`, proved by a per-device
secret), offered only when `isUserVerifyingPlatformAuthenticatorAvailable()` is true. Passkey
requests ask for the device's own store (`authenticatorAttachment: platform`, `hints`), so no
USB security-key prompts appear.

## Consequences
Works on every browser. People type their words on devices without a passkey store, and those
devices stay signed in until sign-out. Supersedes the "passkey required" part of ADR 0010.
