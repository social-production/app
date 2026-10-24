# 0050. Voice calls in direct chats: Holochain direct signals instead of web's call socket

Status: accepted (2026-10-24). Builds on [0043](./0043-direct-messages.md).

## Context
web has one-to-one voice calls from a **direct** chat's header (`web/src/lib/calls/*`): a
full-screen call (ring, ringback, timer, mute, speaker, a "Tap to return" pill), audio over
WebRTC between the two browsers, and signalling (invite, accept, reject, offer, answer, ICE
candidates, hangup) through web-backend's `/calls/ws` socket and Redis. The server also knew
who was online ("Not available right now"), kept one call per chat, checked that both people
are in the direct chat, and logged a "Voice call · 4:12" / "Missed call" line in the chat.

The app has no server in the middle. Holochain 0.7's app API has **direct signals**:
`sendDirectSignal` delivers bytes to other agents' cells on a network, without running a zome
or writing anything; a recipient takes them only if their own chain holds a direct-signal
capability grant, which `grantDirectSignalCapability` writes from the app socket.

## Decision
- **Signalling = direct signals on the inbox** (the base conversation cell, one network for
  everyone). Each person grants it once (`holo/signals.ts acceptSignals`, unrestricted; a
  private grant entry on their chain, remembered per device). Envelopes are web's JSON
  (`callId`, `conversationId` = the direct chat's seed, `type`, `payload`), at most web's 64 KB.
- **The receiver checks what web's server checked** (`services/calls.ts`): a signal counts only
  if its chat is a direct chat in your inbox and the sender (whose key Holochain vouches for) is
  its other member. The name on the ring screen is that person's profile name, never the
  payload's.
- **UI ported from web**: `CallHost` (features/calls, mounted by the root layout), its
  controls (`components/calls`), `session.ts` → `callSession.svelte.ts` + `callSignals.ts`,
  `media.ts` → `services/callMedia.ts` (same `VITE_CALL_STUN_URLS` / `VITE_CALL_TURN_*`),
  `tones.ts`; styles via `styles/PORTS` (`calls.css`). Messages asks for a call through
  `stores/callRequest`, so neither feature imports the other.
- **"Not available right now"**: a node can't tell whether its person has the app open, so the
  callee's device answers an invite with an app-only `ringing` envelope; with none within 15 s
  the caller's screen says "Not available right now" (web's server knew at once).
- **No call lines in the chat yet**: web's `kind: 'call'` message needs a field the
  conversation DNA's `Message` doesn't have. It waits for the next conversation-DNA integrity
  change (as project-chat attachments wait for the project DNA's, ADR 0049).
  (Done in ADR 0051.)

## Consequences
- **UI only**: no zome or DNA change, so deploying resets nothing.
- Anyone on the inbox network may send signals to your node; the UI drops all but your direct
  chats' other members. A flood of junk would cost parsing, not privacy. Signals aren't stored
  on the DHT; an offer reveals the caller's network addresses (ICE candidates) to the callee, as
  on web.
- Audio needs a reachable path between the two devices. With only STUN, calls between some
  networks (symmetric NAT, strict firewalls) fail to connect; web has the same limit until a
  TURN relay is configured (`VITE_CALL_TURN_URLS`).
- A person signed in on two devices rings on both; the first to answer takes the call (the
  other keeps ringing until the caller hangs up), as on web.
- Group calls, video, and a ring when the app is closed (push) are not ported; web has none.

## Addendum (2026-10-24): the ring must not wait on reads
Live calls stopped ringing after the ADR 0051 deploy. Watching the websocket frames showed the
network delivered the invite, but the callee's check of the sender read the chat's whole
summary first, which on a busy node took longer than the caller's 15 s; the caller then gave up
("Not available right now") and the late ring closed at once. Also, a signal sent while the
callee's page reloads is lost (direct signals are best effort). Now:
- **The sender check reads as little as possible** (`services/calls.ts` `findPeer`): a chat's
  seed names its creator and kind, a direct chat has two people, and the seed is in your inbox
  only if you are one of them, so in a chat someone else started the other person is its
  creator. Only the creator's own side reads the chat's members, and the call button passes
  the member it already knows (`rememberPeer`). The inbox is read once and again only on a
  miss.
- **The caller repeats its invite every 2.5 s** until the callee's device answers `ringing`
  (or 15 s pass); the callee answers a repeat again and ignores invites for calls that already
  ended on it.
- Nothing here needs the seed node: two desktop apps signal each other through the relay
  (the bootstrap service), and two people hosted by the same gateway are bridged inside it.
