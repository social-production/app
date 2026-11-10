# 0088. Deactivating an account is a record in the directory, honoured by every reader

Status: **accepted** (owner, 2026-11-10: "with deactivation, I guess the directory? Remember
this app needs to function without a seed node"). Not built yet: it is a directory integrity
change, so it should ride with the next directory reset. Follows [0074](./0074-bot-reviews.md)
(paused accounts replayed by readers) and [`docs/plans/SEEDLESS.md`](../plans/SEEDLESS.md).

## Context
web's Settings → Account has **Deactivate account**: "You will be signed out and cannot sign in
again. Posts, threads, and vouches stay under this username." (password-confirmed). web-backend
marks the user inactive and refuses their sign-ins. The app has no password and no server that
can refuse a sign-in: a person's key works on any node, and the seed node will go away
(SEEDLESS.md). So the rule has to live in the data every node already holds.

## Decision
- **Record:** a `Deactivated` link from the account to itself in the directory, written by the
  account (self only, never deleted), like ADR 0074's `ReReviews`. Linking from the account
  keeps it next to the profile, standing and bot-review links that readers already fetch.
- **Readers honour it** (as they honour Paused, ADR 0074): the profile shows the account as
  deactivated; its future votes and vouches count for nothing, and it leaves quorum
  populations; its earlier posts, threads, vouches and votes stay as they were, as on web.
- **Signing in is refused by every client**, not by a server: after unlocking a key, the UI
  reads the directory and, if the account is deactivated, shows web's message and signs out.
  Gateways refuse it too, for browsers. A modified client could still write, but every honest
  reader ignores what that account does after the link, which is web's effect.
- **Directory writes after it are invalid:** the identity integrity rules reject a directory
  create by an author whose chain already has the `Deactivated` link (read with
  `must_get_agent_activity` from the link back, as `chain.rs` does for rate limits). Project and
  conversation DNAs can't read the directory, so there readers ignore it (above).
- **Confirm step:** web asks for the password; the app asks the person to type their username
  (it has no password) before writing the link.

## Consequences
- Directory integrity change: merging resets the directory network (test data only until
  cutover). Batch it with other directory changes.
- Works with no seed node: any node that holds the directory enforces and displays it.
- Irreversible, as on web: a person who wants back in makes a new account.
