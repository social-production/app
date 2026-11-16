# 0107. Push notifications: a signal from the writer, Web Push from the seed, the OS on desktop

Status: **accepted** (2026-11-16, owner; from the owner's parity list: "Web doesn't have notifications push
either, that needs to be built"). Builds on [0046](./0046-notifications-derived-on-device.md).

## Context
Notifications are derived on each device (ADR 0046): the app reads the projects, chats, follows
and reviews it has open and turns them into web's cards. That needs the app to be running. A
browser tab that is closed, or a desktop app in the tray, learns nothing until it is opened again.
web has no push either (the FastAPI site only has the bell), so this is new.

The recipients are known at the moment something happens: web-backend's `create_notification`
is called with them on every write (the replied-to author, a project's members for a phase vote,
the people asked to vote on a bot review). The writer's coordinator knows the same people.

A browser account lives on a seed node (its cells run there), so a seed always has the
recipient's cells running. A desktop account's cells run on the desktop, only while the app runs.

## Decision
- **The writer sends a signal; the list stays derived.** Each write that makes a card on web
  (reply, mention, phase vote and result, follow, invite, share, role, request, bot review vote
  and result, plan leading) also sends a Holochain remote signal from the writer's coordinator to
  the recipients: `{ kind, title, body, link }`, worded as the card is. A signal is a nudge,
  not a record. Missed signals (recipient offline) are fine, because the Notifications page and
  the bell are still built from the chains as in ADR 0046. Coordinator code only (a new
  `recv_remote_signal` and an unrestricted grant for it, made lazily the first time an app
  starts), so **no network reset**.
- **Browsers: Web Push from the seed.** The gateway listens to the app signals of the agents it
  hosts. Settings → Notifications gets "Push notifications on this device": switching it on asks
  the browser for permission, registers a small service worker (`ui/static/sw.js`) and sends the
  browser's push subscription to `POST /push/subscribe` (signed like `/session/resume`), with the
  categories switched off on that device. The gateway keeps subscriptions in `gateway.sqlite`
  (agent, endpoint, keys, muted categories) and, for each signal to a hosted agent, sends an
  encrypted Web Push (VAPID, RFC 8291) to each of that agent's subscriptions whose category is on.
  Tapping the notification opens its link. `DELETE /push/subscribe` and signing out remove it; a
  410 from the push service removes it too. The VAPID key pair is made on first start and kept in
  `gateway.sqlite`.
- **Desktop: the OS's own notifications.** The desktop app gets the same signals from its own
  conductor and shows them with Tauri's notification plugin, with the same switch and categories.
  Nothing goes through a seed or a push service.
- **No seed:** a desktop still notifies itself while the app runs (signals travel peer to peer).
  A browser account needs a seed to exist at all, so it has push exactly when it has a seed. A
  hosted account whose seed retired it (ADR 0092) loses its subscriptions with it.
- **Privacy:** the push service (Google, Mozilla, Apple) sees only that a message was sent to an
  endpoint; the payload is encrypted to the browser. A second switch, "Show what it's about", is
  on by default; off, the push says only "Something new on Social Production" and the app is
  opened to read it. Items tagged only to a closed community are never pushed with their text.
- **Read state:** a push doesn't mark anything read; opening its link does, as a card tap does.

## Consequences
- Every notifying write gains one `send_remote_signal` call. The recipients are the people a card
  goes to anyway, so the cost is one small message each.
- The gateway gets a push module (subscriptions table, VAPID, a sender with retries) and two
  routes. Railway's seed already reaches the internet, so no network change.
- iPhones get Web Push only for the site added to the Home Screen (iOS 16.4 and later); the
  switch says so there.
- Mobile (Tauri on Android/iOS) would need the platform's push service (FCM/APNs) to wake a
  closed app. That waits for the mobile builds.
- The bot-review vote notification (parity item 1) is built first with its card, then gets its
  signal with the rest.
- Order of work: (1) the signals and the desktop's OS notifications, testable in scenarios;
  (2) the gateway's subscriptions and sender; (3) the service worker and the Settings switch.
