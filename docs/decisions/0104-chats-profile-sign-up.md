# 0104. Messages, conversations, profiles and sign-up in the soft look

Status: **accepted** (2026-11-13, owner, from mockups; 2026-11-14: an "Edit profile" button
on your own profile is fine).

## Context
The screen audit (2026-11-13) found the old boxed look left on Messages and chats, Profile
and Onboarding, and a bug: a long link in a chat message ran past the bubble's right edge.

## Decision
No feature is added or removed.

- **Messages.** "Personal" and "Public" are one soft switch with the unread counts; rows are
  larger, without boxes, unread ones in bold with a count; the phone shows the "Messages"
  heading.
- **A conversation (and project chat).** Rounded bubbles, yours in the brand colour on the
  right; long words and links wrap inside the bubble; a link to something in the app shows as
  a card inside the bubble; the day is a soft pill; the message bar floats as one capsule
  (photo, file, the field, a round send button); the header is a round back button, the face
  and the name at the left, and a bare call button.
- **Profile.** A larger photo over a soft band, the name with its standing chip, the counts,
  and Follow / Message / Vouch with their words (the bot mark stays a round icon). Your own
  profile has "Edit profile", which opens Settings.
- **Sign-up and log-in.** No boxes: the logo, "Join Social Production", a soft Sign up / Log
  in switch, large rounded fields and a full-width button; the three sign-up steps show as a
  progress line; the 24 words are a three-column grid of soft chips.

## Consequences
- UI only: styles in `styles/ux/messages.css`, `chat.css`, `profile.css`, `onboarding.css`,
  with small markup changes in `Welcome`, `SignupFlow` and `ProfileView`.
- Not taken from the mockup yet: one "+" for photo and file, a shared item's card with its
  date and place, "Report as bot" under a ⋯ menu, a squared avatar for group chats.
- `shell.mjs` waits for the new sign-up heading.
