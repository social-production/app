# 0090. The desktop app's account is the recovery phrase, as on a gateway

Status: **accepted** (2026-11-10, cloud session; SEEDLESS step 2). Supersedes the "own agent"
part of [0024](./0024-pair-devices-not-keys.md) for the desktop app; pairing stays for people
who want a second device with its own key.

## Context
The owner wants the app to work with no seed node (docs/plans/SEEDLESS.md). Until now the
desktop app made a **random** agent on first run and opened straight into the app with no
profile, and importing the phrase was "reserved for M7". With no seed node there is no hosted
account to pair with, so a desktop person had no way to make an account with a name, nor to
get their account back on a new computer. Holochain's Tauri runtime (`tauri-plugin-hc`)
already imports a 32-byte seed into the keystore (`Runtime::import_key_seed`), and its windows
can open unbound and be bound to an app later (`rebind_window`).

## Decision
- **The phrase's seed is the agent key on desktop too** (ADR 0022's rule): the same 24 words
  are the same person on any gateway and any device.
- The main window opens **unbound**. The UI asks the app's `account_state` command; with no
  account it shows web's onboarding (Sign up / Log in). Sign up makes 24 words, as in a
  browser, and calls `open_account` (seed, restore = false); Log in calls it with restore =
  true, which installs the hApp with `restore_from_dht` and waits up to 120 s for the chain to
  arrive before starting fresh (ADR 0035's fallback; a gateway waits 45 s because it holds the
  whole network, a new desktop holds nothing). No proof-of-work and no passkeys on desktop:
  both protect a gateway's hosted accounts.
- Sign out uninstalls the app from this device; the chain stays on the network and the words
  bring it back. A reload asks `account_state` again (the plugin's start-up script always
  injects the window's first binding, which is none).
- Apps installed by older builds (a random agent) keep working: `account_state` finds them.

## Consequences
- An account hosted on a gateway and logged in on a desktop runs on two nodes at once, which
  can fork its chain. Until the gateway can **retire** a hosted copy (next: an
  `/account/retire` call the desktop makes with the seed, and the gateway refusing to restore
  a retired agent), the log-in screen on desktop asks people to use the account in one place.
- Signing out of the desktop with no other node holding the chain loses it. CI showed this:
  signing out two seconds after sign-up and logging back in gave the same key but no profile
  (a fresh chain). Next: sign-out waits for, or warns until, the chain is published (e.g. the
  seed node's index or, without one, the readiness count of SEEDLESS step 7 shows it). The
  smoke test waits for the seed node to list the profile before signing out (59 s on
  2026-11-11) and checks the profile comes back after logging in again.
- The desktop UI still reads the feed index from a gateway (`VITE_SP_GATEWAYS`) until SEEDLESS
  step 4 moves the index onto the device.
- Tested in CI: `.github/workflows/desktop.yml` builds the app on Linux, Windows and macOS and
  runs `tests/desktop/smoke.mjs` (sign up, profile, sign out, log in with the words) against
  the real app on Linux through tauri-driver.
