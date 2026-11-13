# 0059. Types evolve through plans; personal services go (after parity)

Status: **accepted** (2026-10-27, owner). Built after parity, with the UX pass (before launch).

## Context
web asks for the type up front (productive, collective service, personal service, event…).
The owner wants content to start generic and become a type through its plans. Since ADR 0057 an
event is a project clone whose charter `mode` is `"event"`, and the leading production plan
already sets the subtype (ADR 0039), so the mode can be decided the same way.

## Decision
- **Everything starts as a proposal**: title, description, support signals and values, no type.
- **The leading approved plan sets the type** when the project leaves the plan phase: a
  production plan makes it productive (software stays a subtype set by that plan), an
  operations plan a collective service, an event plan an event. Plans of every kind may be
  filed in the first plan phase; leadership is decided as today (ADR 0039).
- **Only a return to the plan phase changes the type** (as returns reopen the subtype, ADR 0040).
- **Personal services are removed:** a service one person runs is a collective service nobody
  has joined yet. To decide when building: moving web's availability calendar and service
  requests onto collective services as plan options.
- **Separate types stay:** private events, threads, help requests.
- Until then, parity work must not add rules that fix the type at creation.

## Consequences
- One project-network reset. No directory reset: listings take any kind (ADR 0057), and the
  index reads the live type from each project.
- Needs the "kind profile" refactor first: one descriptor per type (nouns, routes, phases,
  copy) instead of string checks spread through the UI and zomes.

## Addendum (2026-11-14): as built, stage A
The owner approved the build on 2026-11-13: the word is **Proposal**, the project network may
be reset (the app is not released), an operations plan may switch requests on, and changing
request settings later is a vote.

- **Charter mode `proposal`.** The create menu offers Post, Thread, Request, Proposal and
  Private event. A proposal's form has no Type step. Its phases are Proposal → Plan; in Plan
  it takes production, operations and event plans side by side ("Add plan" asks which).
- **One leader across the competing kinds** (`plans::leader(approved, &kinds)`). The phase
  move out of Plan cites it, and the replay (`Applied.mode`) gives the proposal that plan's
  type and continues in that type's next phase. `Applied.leading_mode` is the type the
  leading plan would give now, for the page's "makes this a …" copy. A return to Plan or
  Proposal clears the type and the subtype.
- **The charter never changes.** Validation reads only the charter, so integrity rules that
  need a type let a proposal through as that type (`applied::typed_as`, `may_be`) and the
  coordinator, which replays decisions, refuses the write when the proposal is not that type.
- **Requests are a plan option.** An operations plan may name a request mode (`Plan.requests`:
  calendar, direct or both). When the phase move cites that plan the service takes requests
  in that mode from then on (`Applied.requests`); a later request-settings vote overrides it,
  and a return to Plan clears it with the type. No change to `sp_core` or `sp_service`, so
  the directory and conversation networks keep their hashes.
- **Personal services:** nothing creates one any more. Their rules stay in the zomes until
  stage B so this stage is one reset of the project network only.
- **Stage B (a second project-network reset, before release):** refuse typed charters at
  creation, remove the personal-service rules, and move the scenario tests that still found
  typed projects onto proposals.
