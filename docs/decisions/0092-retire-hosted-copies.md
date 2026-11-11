# 0092. A desktop log-in retires the gateway's hosted copy; sign-out warns while no one else holds the account

Status: **accepted** (2026-11-11, cloud session; the rest of SEEDLESS step 2). Completes
[0090](./0090-desktop-accounts-from-the-phrase.md)'s two open consequences.

## Context
ADR 0090 made the desktop's account the recovery phrase. Logging in on a desktop with an
account a gateway hosts left the account running on two nodes at once, which can fork its
chain (two different next actions). And signing out of a desktop that is the only holder of a
new chain lost it (CI showed this: same key, empty profile).

## Decision
- **Retirement, signed on the device.** When a desktop log-in restores an existing chain from
  the network (`restore_from_dht` settled; never when it had to start a fresh chain, since the
  hosted copy may then hold the only history), `open_account` returns a statement signed with
  the phrase's ed25519 key: `social-production: retire the hosted copy of <agent> at <unix
  secs>` (`native/src/retire.rs`). The UI posts it to every configured gateway's
  `POST /account/retire` (best effort, in the background). No secret leaves the device.
- **The gateway checks and stops hosting** (`services/gateway/src/routes/retire.rs`): the
  signature must verify against the agent key and the time must be within 10 minutes of the
  gateway's. If it hosts the person it sets `persons.retired_at` and uninstalls their app
  (`Node::retire_hosted`). Uninstall rather than disable: a later reclaim must restore the
  newer chain from the network, not resume the stale local one, and Holochain's coordinator
  updates at start only touch installed apps. Rate limited like sign-in.
- **A retired account is not signed in again by accident.** Phrase sign-in or restore answers
  **410** "This account moved to the desktop app…"; the log-in form then offers **Use it here
  instead**, which repeats the restore (with its proof-of-work) with `reclaim: true`: the
  gateway clears `retired_at` and restores the chain from the network. Passkey sign-in
  answers 410 and points to the phrase. Existing browser sessions end on their own: the app
  is gone, so `/session/resume` asks them to sign in again.
- **Sign-out on desktop warns** (Settings → Account) while no other node lists the account:
  today the gauge is a gateway's people index (`listedOnNetwork`); without an answer, or not
  listed, a confirm says the profile and history could be lost and offers "Sign out anyway".
  When the index moves onto devices (SEEDLESS step 4) or the readiness count lands (step 7),
  the gauge follows.

## Consequences
- A person can move from the website to the desktop and back with their words alone.
- Actions the hosted copy wrote but had not yet published when it was retired are lost (the
  desktop restored what the network had). Publishing takes seconds, so this needs a write in
  the same moment as the log-in.
- If no gateway answers at log-in, the hosted copy keeps running until the next desktop
  log-in. A later step can resend the retirement on start (the desktop would need to keep the
  signed statement, which is harmless: it only retires).
- The reverse direction (a desktop account restored on a website) is not blocked: the gateway
  can't know about desktops. The desktop's log-in screen still says to use one place.
- Tested: gateway and native unit tests (signature round trip, the exact statement bytes);
  `tests/e2e/retire.mjs` against a local seed node (wrong key and stale time refused, retire,
  log-in shows the move, "Use it here instead" brings the profile back).
