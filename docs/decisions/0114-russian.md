# 0114. Russian: a typed string catalogue, chosen per device

Status: **accepted** (2026-11-18, owner: "Do a russian translation and ill check it with my wife
tonight"; first open follows the browser or device language; push in the reader's language is fine).
From the owner's parity list: "One language besides English: Russian".

## Context
Every screen's words are written in place in about 320 Svelte files (web is the same; its
Settings › Appearance has a Language row marked "coming soon"). Three kinds of text reach
people:
1. The app's own words: labels, buttons, empty states, errors, the copy in sheets.
2. Words built from data: notification cards ("Ann proposed a phase change"), relative times,
   dates, counts ("3 signed up · 2 needed").
3. Push notifications (ADR 0108). Today the **writer's** device words them and sends the text,
   so a recipient would get the writer's language.

What people write (posts, titles, chats) is never translated.

## Proposal
- **Catalogue (as built):** the English text is the key, gettext style: `t('Home')` is "Home"
  in English and `ru['Home']` in Russian, and anything missing stays English. This let every
  screen be converted mechanically (a script wrapped fixed text between tags, word-holding
  attributes, and text choices in code) instead of inventing thousands of key names.
  `ui/src/lib/i18n/ru.ts` merges `ru/part*.ts` (files stay under 300 lines). Words that are
  also used as values in code (phase names, statuses) are translated where they are shown,
  `{t(stage)}`, never where they are defined. `t(text, vars)` fills `{name}` slots; plurals use `Intl.PluralRules`
  (Russian has one, few and many: `{count, plural, one {# участник} few {# участника} many
  {# участников}}`). No library: the whole thing is a few dozen lines and adds nothing to
  load. Dates, times and numbers go through `Intl` with the chosen locale (the Display
  timezone stays as it is).
- **Choosing the language:** Settings › Appearance › Language (web's "coming soon" row
  becomes real): English or Русский. The first time, the browser's language decides (`ru*` →
  Russian). Kept per device, like the other appearance choices.
- **Push and notices in the recipient's language:** a notice carries its `kind` and the
  values the card needs (names, titles), not finished text. The recipient's device words it
  (as the bell already does from the chains). For Web Push the seed words it: the push
  subscription also stores the device's language, and the gateway keeps the same short
  catalogue for notice kinds (`services/gateway`). Coordinator code and the gateway only: no
  network reset.
- **Who translates:** Claude drafts the Russian; the owner (or someone they choose) reads it
  before it's switched on. Until then the Language row stays "coming soon".
- **How it's built:** one PR per area (shell and Settings first, then feeds and cards, project
  pages, chats, sheets and wizards), each moving that area's strings into the catalogue. A
  check script flags literal text left in `.svelte` templates in areas already moved, so
  nothing new slips back.

## Questions for the owner (answered 2026-11-18)
1. Claude's draft ships; the owner and a native speaker read it on the live site.
2. The first open follows the browser or device language.
3. Push in the recipient's language: fine (not built yet; notices are still worded by the
   sender's device).

## Consequences
- Every UI file is touched once, so the work goes area by area after open UX branches merge.
- New UI copy is written as a catalogue key from then on (CODE_RULES gets the rule).
- The desktop and Android/iOS apps get the same catalogue (same UI build).
