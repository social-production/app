# 0020. Mobile platform findings (M0 platform check)

Status: accepted (2026-10-18)

## Context
ADR 0011 requires phones to become nodes. M0 cross-compiles the embedded conductor
(`tests/mobile-conductor`) for iOS and Android in `.github/workflows/platforms.yml`.

## Findings
- **Desktop (Linux)**: works end to end. The Tauri app's embedded conductor synced with a seed node in M0.
- **Android**: the conductor's encrypted database (SQLCipher) links OpenSSL, which Android
  doesn't provide. Fixed by building vendored OpenSSL for mobile targets.
- **iOS**: blocked upstream. `lair_keystore 0.7.1` and `human-panic` depend on `sysinfo ^0.38`,
  which fails to compile for iOS (`proc_pidinfo`, `RTM_IFINFO2` are macOS-only in `libc`).
  `sysinfo 0.39` has iOS fixes, but it's semver-incompatible, so we can't bump it ourselves.

## Decision
- Keep both checks in CI. iOS runs with `continue-on-error` so its status stays visible.
- Before M7, choose one: (a) Holochain releases a keystore on `sysinfo 0.39` (raise upstream);
  (b) maintain a `[patch]`ed `sysinfo 0.38` fork with the iOS cfg fixes; (c) ship iOS as a
  thin client of the user's own desktop/community node until (a) or (b) lands.
- Android stays on the M7 light-node path.

## Consequences
The iOS phone-as-node goal carries a known upstream risk, and it's tracked here instead of being discovered at M7.
