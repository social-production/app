# Identity

## Agents and the recovery phrase (ADR 0022)

Every person is a Holochain **agent**, an Ed25519 key that signs everything they write.
The agent key is derived from a 32-byte seed, and **the 24-word recovery phrase is that seed**
(BIP-39 words encoding the raw 32 bytes, with no password stretching). The same phrase therefore
gives the same agent on any node or app. Anyone holding the words can act as the person.

Names are not unique. People are shown as `Name · tag`, where the tag is 4 characters of the
agent key (`ui/src/lib/domain/people.ts`).

## Hosted agents (browser / PWA users), ADR 0010 and 0023

The seed node hosts an agent for each person who signs up in a browser. Everything happens in
`services/gateway` (one process with the conductor embedded and no admin port):

The **recovery phrase is always enough** to sign in (ADR 0027). Passkeys are an optional shortcut,
offered only on devices with their own passkey store, and never as a USB security key.

| Step | Browser | Gateway (`/account/*`) |
|---|---|---|
| Sign up | makes 24 words, shows them, asks for 3 back; solves a proof-of-work; sends the 32-byte seed with its own new signing key | `open` (signup): checks proof-of-work + per-IP limit, imports the seed, installs the person's app (`person-<agent>`), grants the device key, returns a session + **device secret** |
| Sign in (phrase) | 24 words → seed | `open` (signin): person must exist on this node; if not, the browser retries as **restore** (same agent, rebuilt from the DHT) |
| Save a passkey (optional) | after sign-in, only if the device can store one | `passkey/start` needs the device secret; the passkey is then bound to the agent |
| Sign in (passkey) | device passkey or phone (QR); never a USB key | `login/*`: credential id → agent, grants a new device key |
| Reload | sends its signing public key | `/session/resume` issues a new token (no prompt) |

The gateway database holds only agent keys, passkey public data and device signing keys.
There are no names, emails or passwords.

**Custodial while hosted:** the seed node holds hosted keys in its keystore (unlocked by
`LAIR_PASSPHRASE`). The recovery phrase is the way out, because the person can take their identity
to any node or app.

## Zome-call signing

Holochain 0.7 requires every app-websocket zome call to be signed. A browser signs with its
**own** key pair, which the gateway granted access with a capability secret. That key cannot sign as the
person's agent; it can only call that person's zomes. Knowing the public half is useless
without the private half. Revoking a device means deleting its grant. Clone cells
(projects in M2, chats in M4) need a grant when they are created.

## Native apps and devices (ADR 0021, 0024)

The desktop (later phone) app runs its own conductor with its **own** agent, created on the
device. One agent must never run on two conductors at once, because its chain would fork. So:

- **Pair, don't share:** on the app, Profile → Devices → paste the main account's key, which sends a
  pairing request; confirm it on the main account. Each side writes half of a `DeviceLink`, and a
  pairing counts only when both halves exist.
- Importing the phrase into the app is reserved for **moving off** a hosted node (M7), at which
  point the hosted copy is retired.

## Signup protection (ADR 0026)

Proof-of-work (`POW_BITS`, about a few seconds) plus a per-IP daily limit (`SIGNUPS_PER_IP_PER_DAY`).
New accounts start at **New** standing (no vote) until vouched, so bot accounts carry no weight.
