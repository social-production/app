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
