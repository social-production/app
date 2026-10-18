# Identity

## Agents

Every user is a Holochain **agent**: an Ed25519 key that signs everything they write.
There are no usernames on the network, only keys and the `Profile` entries those keys author.

## Hosted agents (PWA / browser users), ADR 0010

Browsers cannot run a conductor, so the seed node hosts an agent for each PWA user:

1. **Sign-up.** The gateway generates a new seed and derives the agent key into the conductor's
   lair keystore. It installs the hApp for that agent and shows the **recovery phrase** once.
   The user registers a **passkey** (WebAuthn) bound to that agent.
2. **Log in.** The passkey assertion is verified by the gateway, which issues a short-lived
   app-websocket token scoped to that agent's app. The UI connects with it.
3. **No passwords or emails** are stored. The gateway stores only passkey public keys mapped to
   installed-app ids.

### Zome-call signing (how a browser talks to a hosted cell)

Holochain 0.7 requires every app-websocket zome call to be signed. The browser never holds
the agent key. Instead:

1. The browser generates its own Ed25519 **signing keypair** (`generateSigningKeyPair`) and
   keeps it in IndexedDB.
2. `POST /session { signingKey }`: the gateway (after login, from M1) creates an **assigned
   capability grant** on each of the user's cells for that public key with a random secret
   (`grant_zome_call_capability`), and issues a single-use app-websocket token.
3. The response `{ appWsUrl, token, capSecret, cellIds }` lets the UI call
   `setSigningCredentials` per cell and connect to `/ws`.

Revoking a device means deleting its grant. Clone cells (projects, chats) get a grant when they are created.
In M0 there is one shared demo agent and no login.

This is **custodial** while hosted: the seed node could sign as the user. The mitigation is
portability. With the recovery phrase, the user restores the same agent key on any node
(desktop app, phone app, community seed node) and their identity and history move with them.
The UI will nudge hosted users to install the app.

## Native agents (desktop / phone apps)

The key is generated and kept on the device (in the embedded lair keystore). The app can show the
recovery phrase, or import one to restore a hosted identity.

## Open items (M1)

- Exact seed/recovery format: Holochain seed bundles (`hc-seed-bundle`) vs BIP-39 words wrapping them.
- Passkey recovery when a phone is lost but the phrase is kept: re-register a passkey from the phrase.
- Multi-device: the same agent on two devices is unsafe (source chain forks), so a second device
  gets its own agent linked to the first via a signed `DeviceLink`.
