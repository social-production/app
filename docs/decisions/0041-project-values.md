# 0041. Project values are records; prominent ones become plan criteria

Status: accepted (2026-10-19). See docs/plans/M3-governance.md. Builds on
[0039](./0039-project-plans.md).

## Context
On web, the Proposal phase's work is values: a member adds one ("What should this achieve?",
up to 160 characters), and members rate how important each is, 1 ("Unnecessary") to 10
("Required"), changing their vote at will (web-backend `add_project_value`,
`vote_project_value_importance`). Values can be added and rated only while the project is in
Proposal; a value already listed (ignoring case) is refused; only established voters rate.
Every value averaging 5 or more becomes a plan criterion, "How well does this plan satisfy
"…"?", and a plan's average rating is the mean over every criterion it was rated on
(`plan_criteria.py`, `plan_average_rating`).

## Decision
- **`Value { label, membership }`** (linked from a "values" anchor) and
  **`Importance { value, membership, importance }`** (linked from the value) are new
  `project_integrity` entries. Integrity checks the label (1–160 characters, trimmed), the 1–10
  range, that an importance vote is on a `Value` record, and that the author was a member.
- **The latest importance vote per member counts**, and only from members with Full standing
  when they voted (web's `ensure_established_voter`, checked at read time as votes are).
- **Proposal phase only, and no duplicates** are coordinator checks with web's messages. Phase
  is derived state, so integrity can't check it; plans' "matching phase" rule works the same way
  (ADR 0039). Reads keep the earliest of two values with the same label (`label_key`: case and
  spacing ignored), so a duplicate written by a modified app never shows or counts twice.
- **Value criteria.** A plan `Rating` may name `value:<value action hash>` for any value of the
  project (integrity checks it is a `Value`); the plan board shows the rubric plus the values
  averaging 5 or more, deduplicated by label. A plan's score now averages every criterion it was
  rated on, value criteria included, as web's does. The rules live in `sp_governance::values`.

## Consequences
- New entry and link types change the project DNA hash: projects were reset again (people stay).
- A value's prominence can change after plans were rated; ratings on a value that stops being
  prominent still count towards the plan's score, as on web.
- A modified coordinator could add or rate values outside Proposal. They would show, but values
  change no decision by themselves: they only add plan criteria, which rank already-approved plans.
